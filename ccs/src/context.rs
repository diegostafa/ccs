use std::collections::HashMap;

use super::ast::{Program, Statement};
use super::process::Process;
use crate::lts::Lts;

#[derive(Debug, Clone, Default)]
pub struct Context {
    constants: HashMap<String, Process>,
}
impl Context {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn constants(&self) -> &HashMap<String, Process> {
        &self.constants
    }
    pub fn bind_process(&mut self, name: String, p: Process) {
        self.constants.insert(name, p);
    }
    pub fn get_process(&self, name: &str) -> Option<&Process> {
        self.constants.get(name)
    }
    pub fn process_to_const(&self, p: &Process) -> Option<Process> {
        self.name_of(p).map(Process::constant)
    }
    pub fn name_of(&self, p: &Process) -> Option<&str> {
        self.constants
            .iter()
            .find(|(_, process)| *process == p)
            .map(|(id, _)| id.as_str())
    }
    pub fn to_lts(&self) -> Lts {
        self.get_process("main").unwrap().clone().derive_lts(self)
    }
}
impl From<Program> for Context {
    fn from(value: Program) -> Self {
        let mut ctx = Self::default();
        for stmt in value.0 {
            match stmt {
                Statement::DefConstant(name, def) => ctx.bind_process(name, def.clone()),
            }
        }
        ctx
    }
}
impl From<String> for Context {
    fn from(value: String) -> Self {
        Self::from(Program::parse(&value).unwrap())
    }
}
