use std::str::FromStr;

use anyhow::{anyhow, Result};
use darklua_core::rules::Rule;
use full_moon::{ast::Ast, visitors::VisitorMut};

use crate::modifiers;

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
    FullMoonVisitor(Box<dyn VisitorMutWrapper>),
}

impl FromStr for Modifier {
	type Err = anyhow::Error;

	fn from_str(s: &str) -> Result<Self> {
		let modifier = match s {
			modifiers::REMOVE_GENERALIZED_ITERATION_MODIFIER_NAME => {
				Modifier::DarkluaRule(Box::<modifiers::RemoveGeneralizedIteration>::default())
			}
			modifiers::BIT32_CONVERTER_MODIFIER_NAME => Modifier::FullMoonVisitor(Box::new(
				modifiers::Bit32Converter::default(),
			)
				as Box<dyn VisitorMutWrapper>),
			_ => Modifier::DarkluaRule(
				s
					.parse::<Box<dyn Rule>>()
					.map_err(|err| anyhow!(err))?,
			),
		};

		Ok(modifier)
	}
}
