use std::{path::PathBuf, str::FromStr};

use anyhow::{anyhow, Result, Error};
use darklua_core::{rules::{self, bundle::BundleRequireMode}, BundleConfiguration, Configuration, GeneratorParameters, Options, Resources};
use indexmap::IndexMap;
use tokio::fs;

use crate::{manifest::Manifest, modifier::Modifier, polyfill::Polyfill, TargetVersion};

pub const DAL_GLOBAL_IDENTIFIER_PREFIX: &str = "DAL_";

pub const DEFAULT_LUAU_TO_LUA_MODIFIERS: [&str; 7] = [
    "remove_interpolated_string",
    "remove_compound_assignment",
    "remove_types",
    "remove_if_expression",
    "remove_continue",
    "remove_redeclared_keys",
    "remove_generalized_iteration",
];

pub const DEFAULT_OPTIMIZING_MODIFIERS: [&str; 11] = [
    "remove_unused_variable",
    "remove_unused_while",
    "remove_unused_if_branch",
    "remove_spaces",
    "remove_nil_declaration",
    "remove_function_call_parens",
    "remove_empty_do",
    "remove_comments",
    "convert_index_to_field",
    "compute_expression",
    "filter_after_early_return",
];

#[inline]
fn modifiers_from_index(modifiers: &IndexMap<String, bool>) -> Result<Vec<Modifier>> {
    modifiers.iter()
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
    target_version: TargetVersion
}

impl Default for Transpiler {
    fn default() -> Self {
        Self {
            modifiers: default_modifiers_index(),
            polyfill: None,
            extension: None,
            target_version: TargetVersion::Default
        }
    }
}

impl Transpiler {
    pub fn with_optimizing_modifiers(mut self) -> Self {
        for name in DEFAULT_OPTIMIZING_MODIFIERS {
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
        if manifest.auto_optimize {
            self = self.with_optimizing_modifiers();
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

    pub fn with_polyfill(mut self, polyfill: Polyfill) -> Self {
        self.polyfill = Some(polyfill);
        self
    }

    async fn private_process(&self, input: PathBuf, output: PathBuf, additional_modifiers: Option<&mut Vec<Modifier>>, bundle: bool) -> Result<()> {
        let resources = Resources::from_file_system();

        let mut modifiers = Vec::new();
        if let Some(additional_modifiers) = additional_modifiers {
            modifiers.append(additional_modifiers);
        }
        modifiers.append(&mut modifiers_from_index(&self.modifiers)?);

        let (rules, mut fullmoon_visitors) = modifiers.into_iter().fold(
            (Vec::new(), Vec::new()),
            |(mut rules, mut fullmoon_visitors), modifier| {
                match modifier {
                    Modifier::DarkluaRule(darklua_rule) => rules.push(darklua_rule),
                    Modifier::FullMoonVisitor(fullmoon_visitor) => {
                        fullmoon_visitors.push(fullmoon_visitor)
                    }
                }
                (rules, fullmoon_visitors)
            },
        );

        let mut options = Options::new(input).with_configuration({
            let mut config = Configuration::empty()
                .with_generator(GeneratorParameters::RetainLines);

            if bundle {
                config = config.with_bundle_configuration(BundleConfiguration::new(
                    BundleRequireMode::from_str("path")
                        .map_err(|e| Error::msg(e.to_string()))?
                ));
            }

            rules.into_iter().fold(config, |config, rule| config.with_rule(rule))
        });
        options = options.with_output(&output);

        let result = darklua_core::process(&resources, options);

        let success_count = result.success_count();
        if result.has_errored() {
            let error_count = result.error_count();
            eprintln!(
                "{}{} error{} happened:",
                if success_count > 0 { "but " } else { "" },
                error_count,
                if error_count > 1 { "s" } else { "" }
            );

            result.errors().for_each(|error| eprintln!("-> {}", error));

            return Err(anyhow!("darklua process was not successful"));
        }

        let extension = &self.extension;
        if fullmoon_visitors.is_empty() {
            if let Some(extension) = extension {
                for mut path in result.into_created_files() {
                    path.set_extension(extension);
                }
            }
        } else {
            for path in result.into_created_files() {
                let code = fs::read_to_string(&path).await?;
                let mut ast = full_moon::parse_fallible(code.as_str(), (&self.target_version).to_lua_version().clone())
                    .into_result()
                    .map_err(|errors| anyhow!("{:?}", errors))?;

                for visitor in &mut fullmoon_visitors {
                    ast = visitor.visit_ast_boxed(ast);
                }

                fs::write(
                    if let Some(extension) = extension {
                        path.with_extension(extension)
                    } else {
                        path
                    },
                    ast.to_string()
                ).await?;
            }
        }

        Ok(())
    }

    pub async fn process(&self, input: PathBuf, output: PathBuf) -> Result<()> {
        self.private_process(input, output, None, false).await?;
        if let Some(polyfill) = &self.polyfill {
            let path = polyfill.path();
            let config = polyfill.config();
            // needed additional modifiers: inject_global_value
            let mut additional_modifiers: Vec<Modifier> = Vec::new();
            for (key, value) in config.defaults() {
                let mut identifier = key.to_string();
                identifier.push_str(DAL_GLOBAL_IDENTIFIER_PREFIX);
                let inject_global_value = rules::InjectGlobalValue::boolean(identifier, *value);
                additional_modifiers.push(
                    Modifier::DarkluaRule(Box::new(inject_global_value))
                );
            }
            self.private_process(path.to_owned(), path.join("dist"), Some(&mut additional_modifiers), true).await?;
        }
        Ok(())
    }
}
