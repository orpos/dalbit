use std::{collections::HashSet, path::PathBuf};

use anyhow::{anyhow, Result};
use full_moon::ast::{Ast, Expression, Field, LastStmt};
use tokio::fs;

use crate::TargetVersion;

pub enum ParseTarget {
    FullMoonAst(Ast),
    File(PathBuf, TargetVersion),
}

pub(crate) async fn parse_file(path: &PathBuf, target_version: &TargetVersion) -> Result<Ast> {
    let code = fs::read_to_string(&path).await?;
    let ast = full_moon::parse_fallible(code.as_str(), target_version.to_lua_version().clone())
        .into_result()
        .map_err(|errors| anyhow!("full_moon parsing error: {:?}", errors))?;

    Ok(ast)
}

pub async fn get_exports_from_last_stmt(target: &ParseTarget) -> Result<Option<HashSet<String>>> {
    let ast = match target {
        ParseTarget::FullMoonAst(ast) => ast,
        ParseTarget::File(path, target_version) => &parse_file(path, target_version).await?,
    };
    let block = ast.nodes();

    if let Some(exports) = block
        .last_stmt()
        .and_then(|last_stmt| match last_stmt {
            LastStmt::Return(return_stmt) => return_stmt.returns().first(),
            _ => None,
        })
        .and_then(|first_return| match first_return.value() {
            Expression::TableConstructor(table_constructor) => Some(table_constructor),
            _ => None,
        })
        .map(|table_constructor| {
            let mut exports: HashSet<String> = HashSet::new();
            for field in table_constructor.fields() {
                if let Some(new_export) = match field {
                    Field::ExpressionKey {
                        brackets: _,
                        key,
                        equal: _,
                        value: _,
                    } => {
                        if let Expression::String(string_token) = key {
                            Some(string_token.to_string())
                        } else {
                            None
                        }
                    }
                    Field::NameKey {
                        key,
                        equal: _,
                        value: _,
                    } => Some(key.to_string()),
                    _ => None,
                } {
                    exports.insert(new_export);
                }
            }
            exports
        })
    {
        return Ok(Some(exports));
    }

    Ok(None)
}
