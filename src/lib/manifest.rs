use anyhow::Result;
use darklua_core::GeneratorParameters;
use full_moon::LuaVersion;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf};
use tokio::fs;

pub const DEFAULT_MANIFEST_PATH: &str = "dal.toml";

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

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TargetVersion {
    Lua51,
    Lua52,
    Lua53,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Manifest {
    output: Option<PathBuf>,
    input: Option<Vec<PathBuf>>,
    file_extension: String,
    target_version: TargetVersion,
    auto_optimize: bool,
    #[serde(default, deserialize_with = "crate::serde_utils::string_or_struct")]
    generator: GeneratorParameters,
    modifiers: HashMap<String, bool>,
    libs: HashMap<String, bool>,
}

impl Default for Manifest {
    fn default() -> Self {
        Self {
            output: None,
            input: None,
            file_extension: "lua".to_owned(),
            target_version: TargetVersion::Lua53,
            auto_optimize: true,
            generator: GeneratorParameters::RetainLines,
            modifiers: HashMap::new(),
            libs: HashMap::new(),
        }
    }
}

impl Manifest {
    pub(crate) fn add_default_modifiers(&mut self) {
        for modifier_name in DEFAULT_LUAU_TO_LUA_MODIFIERS {
            self.insert_modifier(modifier_name.to_owned(), true);
        }
        if self.auto_optimize {
            for modifier_name in DEFAULT_OPTIMIZING_MODIFIERS {
                self.insert_modifier(modifier_name.to_owned(), true);
            }
        }
    }

    pub async fn from_file(path: impl Into<PathBuf>) -> Result<Self> {
        let content = fs::read_to_string(path.into()).await?;

        Ok(toml::from_str(content.as_str())?)
    }

    pub async fn write(&self, path: impl Into<PathBuf>) -> Result<()> {
        fs::write(path.into(), toml::to_string(self)?).await?;

        Ok(())
    }

    pub fn insert_modifier(&mut self, modifier_name: String, enabled: bool) {
        let enabled = if let Some(&old_enabled) = self.modifiers.get(&modifier_name) {
            old_enabled && enabled
        } else {
            enabled
        };
        self.modifiers.insert(modifier_name, enabled);
    }

    pub fn contains_rule(&self, modifier_name: String) -> bool {
        self.modifiers.contains_key(&modifier_name)
    }

    pub fn modifiers(&self) -> &HashMap<String, bool> {
        &self.modifiers
    }

    pub fn target_version(&self) -> LuaVersion {
        match &self.target_version {
            TargetVersion::Lua51 => LuaVersion::lua51(),
            TargetVersion::Lua52 => LuaVersion::lua52(),
            TargetVersion::Lua53 => LuaVersion::lua53(),
        }
    }

    pub fn generator(&self) -> &GeneratorParameters {
        &self.generator
    }

    pub fn extension(&self) -> &str {
        &self.file_extension.as_str()
    }
}
