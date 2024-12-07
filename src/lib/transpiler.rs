use std::{collections::HashMap, path::PathBuf, str::FromStr};

use anyhow::{anyhow, Result};
use darklua_core::{
    rules::{self, bundle::BundleRequireMode},
    BundleConfiguration, Configuration, GeneratorParameters, Options, Resources,
};
use full_moon::ast::Ast;
use indexmap::IndexMap;
use tokio::fs;

use crate::{
    injector::Injector, manifest::Manifest, modifiers::Modifier, polyfill::Polyfill, utils,
    TargetVersion,
};

pub const DEFAULT_LUAU_TO_LUA_MODIFIERS: [&str; 8] = [
    "remove_interpolated_string",
    "remove_compound_assignment",
    "remove_types",
    "remove_if_expression",
    "remove_continue",
    "remove_redeclared_keys",
    "remove_generalized_iteration",
    "remove_number_literals",
];

pub const DEFAULT_MINIFYING_MODIFIERS: [&str; 11] = [
    "remove_spaces",
    "remove_nil_declaration",
    "remove_function_call_parens",
    "remove_comments",
    "convert_index_to_field",
    "compute_expression",
    "filter_after_early_return",
    "remove_unused_variable",
    "remove_unused_while",
    "remove_unused_if_branch",
    "remove_empty_do",
];

#[inline]
fn modifiers_from_index_map(modifiers: &IndexMap<String, bool>) -> Result<Vec<Modifier>> {
    modifiers
        .iter()
        .filter_map(|(key, &value)| {
            if value {
                Some(Modifier::from_str(key.as_str()))
            } else {
                None
            }
        })
        .collect()
}

#[inline]
fn default_modifiers_index() -> IndexMap<String, bool> {
    let mut modifiers: IndexMap<String, bool> = IndexMap::new();
    for name in DEFAULT_LUAU_TO_LUA_MODIFIERS {
        modifiers.insert(name.to_owned(), true);
    }
    modifiers
}

/// A transpiler that transforms luau to lua
pub struct Transpiler {
    modifiers: IndexMap<String, bool>,
    polyfill: Option<Polyfill>,
    extension: Option<String>,
    target_version: TargetVersion,
    injected_polyfill_name: Option<String>,
    polyfill_config: Option<HashMap<String, bool>>
}

impl Default for Transpiler {
    fn default() -> Self {
        Self {
            modifiers: default_modifiers_index(),
            polyfill: None,
            extension: None,
            target_version: TargetVersion::Default,
            injected_polyfill_name: None,
            polyfill_config: None
        }
    }
}

impl Transpiler {
    pub fn with_minifying_modifiers(mut self) -> Self {
        for name in DEFAULT_MINIFYING_MODIFIERS {
            self.modifiers.insert(name.to_owned(), true);
        }
        self
    }

    pub fn with_modifiers(mut self, modifiers: &IndexMap<String, bool>) -> Self {
        for (key, value) in modifiers {
            let value = if let Some(&default_value) = self.modifiers.get(key) {
                default_value && *value
            } else {
                *value
            };
            self.modifiers.insert(key.to_owned(), value);
        }
        self
    }

    pub fn with_manifest(mut self, manifest: &Manifest) -> Self {
        self = self.with_modifiers(manifest.modifiers());
        if manifest.minify {
            self = self.with_minifying_modifiers();
        }
        if let Some(extension) = manifest.extension() {
            self = self.with_extension(extension);
        }
        self.target_version = manifest.target_version().clone();
        self
    }

    pub fn with_extension(mut self, extension: impl Into<String>) -> Self {
        self.extension = Some(extension.into());
        self
    }

    pub fn with_polyfill(
        mut self,
        polyfill: Polyfill,
        new_injected_polyfill_name: Option<String>,
    ) -> Self {
        self.polyfill = Some(polyfill);
        self.injected_polyfill_name = new_injected_polyfill_name;
        self
    }

    #[inline]
    async fn parse_file(&self, path: &PathBuf) -> Result<Ast> {
        utils::parse_file(path, &self.target_version).await
    }


}
