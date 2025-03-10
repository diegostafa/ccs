use std::collections::{HashMap, HashSet};

use ccs::context::Context as ContextCcs;
use lalrpop_util::lexer::Token;
use lalrpop_util::ParseError;

use super::ast::{Command, Program, Statement};
use super::process::Process;
use super::values::{AExpr, BExpr, Value};
use crate::utils::permute;
use crate::values::Enum;

#[derive(Debug, Clone, Default)]
pub struct Context {
    main: String,
    constants: HashMap<String, (Vec<String>, Process)>,
    enums: HashMap<String, Vec<(String, Vec<String>)>>,
    aliases: HashMap<String, String>,
    int_bounds: (u32, u32),
    cached_values: HashMap<String, Vec<Value>>,
}
impl Context {
    const INT_TY: &'static str = "int";
    const BOOL_TY: &'static str = "bool";
    const ANY_TY: &'static str = "any";

    pub fn types(&self) -> Vec<String> {
        [Self::INT_TY.to_string(), Self::BOOL_TY.to_string()]
            .into_iter()
            .chain(self.enums.keys().cloned())
            .chain(self.aliases.keys().cloned())
            .collect()
    }
    pub fn type_of<'a>(&self, v: &'a Value) -> &'a str {
        match v {
            Value::AExpr(_) => Self::INT_TY,
            Value::BExpr(_) => Self::BOOL_TY,
            Value::Enum(Enum::Var(_)) => panic!(),
            Value::Enum(Enum::Lit(ty, ..)) => ty.as_str(),
            Value::Any(_) => Self::ANY_TY,
        }
    }

    pub fn enums(&self) -> &HashMap<String, Vec<(String, Vec<String>)>> {
        &self.enums
    }
    pub fn constants(&self) -> &HashMap<String, (Vec<String>, Process)> {
        &self.constants
    }

    pub fn to_ccs(&self) -> ContextCcs {
        self.constants
            .get(&self.main)
            .unwrap_or_else(|| panic!("Main process \"{}\" not found", self.main));

        let mut ccs = ContextCcs::default();
        ccs.set_main(self.main.clone());
        Process::constant(&self.main, vec![]).to_ccs(self, &mut ccs, &mut HashSet::new());
        ccs
    }
    pub fn bind_enum(&mut self, ty: String, tags: Vec<(String, Vec<String>)>) {
        assert_ne!(ty, Self::ANY_TY);
        assert!(tags
            .iter()
            .flat_map(|(_, fields)| fields)
            .all(|field| *field != ty && field != Self::ANY_TY
                || field == Self::INT_TY
                || field == Self::BOOL_TY
                || self.enums.contains_key(field)
                || self.aliases.contains_key(field)));
        self.enums.insert(ty, tags);
    }
    pub fn bind_process(&mut self, name: String, p: (Vec<String>, Process)) {
        self.constants.insert(name, p);
    }
    pub fn bind_alias(&mut self, alias: String, ty: String) {
        self.aliases.insert(alias, ty);
    }
    pub fn get_process(&self, name: &str) -> Option<&(Vec<String>, Process)> {
        self.constants.get(name)
    }
    pub fn set_bounds(&mut self, (min, max): (u32, u32)) {
        assert!(min < max);
        self.int_bounds = (min, max);
    }
    pub fn set_main(&mut self, main: String) {
        self.main = main;
    }
    pub fn bounds(&self) -> (u32, u32) {
        self.int_bounds
    }
    pub fn values(&self) -> Vec<Value> {
        assert!(!self.cached_values.is_empty());
        self.cached_values.values().flatten().cloned().collect()
    }
    pub fn values_of(&self, ty: &str) -> Vec<Value> {
        if let Some(vals) = self.cached_values.get(ty) {
            return vals.clone();
        }
        if let Some(ty) = self.aliases.get(ty) {
            return self.values_of(ty);
        }
        if ty == Self::BOOL_TY {
            return [true, false].map(|v| Value::BExpr(BExpr::Lit(v))).to_vec();
        }
        if ty == Self::INT_TY {
            return (self.int_bounds.0..self.int_bounds.1)
                .map(|v| Value::AExpr(AExpr::Lit(v)))
                .collect();
        }
        if let Some(tags) = self.enums.get(ty) {
            let mut values = vec![];
            for (tag, fields) in tags {
                if fields.is_empty() {
                    values.push(Value::Enum(Enum::Lit(ty.to_string(), tag.clone(), vec![])));
                    continue;
                }
                let tag_field_vals = fields.iter().map(|f| self.values_of(f)).collect();
                for perm in permute(tag_field_vals) {
                    values.push(Value::Enum(Enum::Lit(ty.to_string(), tag.clone(), perm)));
                }
            }
            return values;
        }
        panic!("{ty} is not a valid type");
    }
    fn gen_values(&mut self) {
        self.cached_values.clear();
        for ty in self.types() {
            self.cached_values
                .insert(ty.to_string(), self.values_of(&ty));
        }
    }
}
impl From<Program> for Context {
    fn from(value: Program) -> Self {
        let mut ctx = Self::default();
        for stmt in value.0 {
            match stmt {
                Statement::DefConstant(name, def) => ctx.bind_process(name, def.clone()),
                Statement::DefEnum(name, tags) => ctx.bind_enum(name, tags.clone()),
                Statement::DefAlias(alias, ty) => ctx.bind_alias(alias, ty),
                Statement::Exec(cmd) => match cmd {
                    Command::SetBounds(min, max) => ctx.set_bounds((min, max)),
                    Command::SetMain(main) => ctx.set_main(main),
                },
            }
        }
        ctx.gen_values();
        ctx
    }
}
impl<'a> TryFrom<&'a str> for Context {
    type Error = ParseError<usize, Token<'a>, &'static str>;
    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        Ok(Self::from(Program::try_from(value)?))
    }
}
