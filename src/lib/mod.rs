use darklua_core::rules::{self, Rule};
use darklua_core::{Configuration, GeneratorParameters, Options, Resources};
use full_moon;
use full_moon::ast::Ast;
use full_moon::visitors::VisitorMut;
use tokio::fs;
use std::collections::HashMap;
use std::path::PathBuf;
use anyhow::{anyhow, Result};

mod modifiers;
pub mod polyfill;
pub mod manifest;
pub(crate) mod serde_utils;

use manifest::Manifest;

pub trait VisitorMutWrapper {
	fn visit_ast_boxed(&mut self, ast: Ast) -> Ast;
}

impl<T: VisitorMut> VisitorMutWrapper for T {
	fn visit_ast_boxed(&mut self, ast: Ast) -> Ast {
		self.visit_ast(ast)
	}
}

pub enum Modifier {
	DarkluaRule(Box<dyn Rule>),
	FullMoonVisitor(Box<dyn VisitorMutWrapper>)
}

pub fn get_modifier_by_name(string: &str) -> Result<Modifier> {
	let modifier = match string {
		modifiers::REMOVE_GENERALIZED_ITERATION_MODIFIER_NAME =>
			Modifier::DarkluaRule(
				Box::<modifiers::RemoveGeneralizedIteration>::default()
			),
		modifiers::BIT32_CONVERTER_MODIFIER_NAME =>
			Modifier::FullMoonVisitor(
				Box::new(modifiers::Bit32Converter::default()) as Box<dyn VisitorMutWrapper>,
			),
		_ =>
			Modifier::DarkluaRule(
				string.parse::<Box<dyn Rule>>().map_err(|err| anyhow!(err))?
			)
	};

	Ok(modifier)
}

fn get_modifiers_from_hashmap(map: &HashMap<String, bool>) -> anyhow::Result<Vec<Modifier>> {
	map
		.iter()
		.filter_map(|(key, &value)| if value { Some(get_modifier_by_name(key.as_str())) } else { None })
		.collect()
}

pub async fn process(mut manifest: Manifest, inputs: Vec<PathBuf>, output: PathBuf) -> Result<()> {
	manifest.add_default_modifiers();

	let resources = Resources::from_file_system();

	for input in inputs {
		let modifiers = get_modifiers_from_hashmap(manifest.modifiers())?;
		let (rules, mut fullmoon_visitors) = modifiers.into_iter().fold(
			(Vec::new(), Vec::new()),
			|(mut rules, mut fullmoon_visitors), modifier| {
				match modifier {
					Modifier::DarkluaRule(darklua_rule) =>
						rules.push(darklua_rule),
					Modifier::FullMoonVisitor(fullmoon_visitor) =>
						fullmoon_visitors.push(fullmoon_visitor),
				}
				(rules, fullmoon_visitors)
			},
		);

		let mut options = Options::new(input).with_configuration({
			rules.into_iter().fold(
				Configuration::empty()
					.with_generator(manifest.generator().clone()),
				|config, rule| config.with_rule(rule)
			)
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

		let file_extension = manifest.extension();
		if fullmoon_visitors.is_empty() {
			for mut path in result.into_created_files() {
				path.set_extension(file_extension);
			}
		} else {
			for path in result.into_created_files() {
				let code =  fs::read_to_string(&path).await?;
				let mut ast = full_moon::parse_fallible(code.as_str(), manifest.target_version())
					.into_result()
					.map_err(|errors| anyhow!("{:?}", errors))?;

				for visitor in &mut fullmoon_visitors {
					ast = visitor.visit_ast_boxed(ast);
				}

				fs::write(path.with_extension(file_extension), ast.to_string()).await?;
			}
		}
	}

	Ok(())
}
