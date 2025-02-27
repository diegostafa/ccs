use std::fmt::Display;

use lalrpop_util::lexer::Token;
use lalrpop_util::{lalrpop_mod, ParseError};

use super::process::Process;

lalrpop_mod!(pub ccs);

pub struct Program(pub Vec<Statement>);
impl<'a> TryFrom<&'a str> for Program {
    type Error = ParseError<usize, Token<'a>, &'static str>;
    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        ccs::ProgramNodeParser::new().parse(value)
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
    Exec(Command),
}
impl Display for Statement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Statement::DefConstant(name, p) => {
                write!(f, "fn {name}() {{ {p} }}")
            }
            Statement::Exec(c) => write!(f, "#![{c:?}]"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Command {
    SetMain(String),
}
