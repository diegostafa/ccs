use std::fmt::Display;

use lalrpop_util::lexer::Token;
use lalrpop_util::{lalrpop_mod, ParseError};

use super::process::Process;

lalrpop_mod!(pub ccs);

pub struct Program(pub Vec<Statement>);
impl Program {
    pub fn parse(content: &str) -> Result<Self, ParseError<usize, Token<'_>, &'static str>> {
        ccs::ProgramNodeParser::new().parse(content)
    }
}
impl Display for Program {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for stmt in &self.0 {
            writeln!(f, "{stmt}")?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum Statement {
    DefConstant(String, Process),
}

impl Display for Statement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Statement::DefConstant(name, p) => {
                write!(f, "fn {name}() {{ {p} }}")
            }
        }
    }
}
