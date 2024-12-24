use std::fmt::{Debug, Display};

use itertools::Itertools;
use lalrpop_util::lexer::Token;
use lalrpop_util::{lalrpop_mod, ParseError};

use super::process::Process;

lalrpop_mod!(pub ccs_vp);

pub struct Program(pub Vec<Statement>);
impl Program {
    pub fn parse(content: &str) -> Result<Self, ParseError<usize, Token<'_>, &'static str>> {
        ccs_vp::ProgramNodeParser::new().parse(content)
    }
}
impl Display for Program {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for cmd in &self.0 {
            writeln!(f, "{cmd}")?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum Command {
    SetBounds(u32, u32),
}

#[derive(Debug, Clone)]
pub enum Statement {
    DefConstant(String, (Vec<String>, Process)),
    DefEnum(String, Vec<(String, Vec<String>)>),
    DefAlias(String, String),
    Exec(Command),
}
impl Display for Statement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Statement::DefConstant(name, (v, p)) => {
                write!(f, "fn {name}({}) {{ {p} }}", v.join(","))
            }
            Statement::DefEnum(name, tags) => {
                let tags = tags
                    .iter()
                    .map(|(tag, fields)| format!("{tag}({})", fields.join(",")))
                    .join(",");
                write!(f, "enum {name} {{ {tags} }}")
            }
            Statement::DefAlias(alias, ty) => write!(f, "type {alias} = {ty};"),
            Statement::Exec(c) => write!(f, "#![{c:?}] "),
        }
    }
}
