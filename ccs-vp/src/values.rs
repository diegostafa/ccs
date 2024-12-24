use std::fmt::Display;

use itertools::Itertools;

use super::context::Context;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    AExpr(AExpr),
    BExpr(BExpr),
    Enum(String, String, Vec<Value>),
    Var(String),
}
impl Value {
    pub fn eval(&self, ctx: &Context) -> Self {
        match self {
            Value::AExpr(e) => Value::AExpr(AExpr::Lit(e.eval(ctx))),
            Value::BExpr(e) => Value::BExpr(BExpr::Lit(e.eval(ctx))),
            Value::Enum(..) | Value::Var(..) => self.clone(),
        }
    }
    pub fn try_replace(&mut self, var: &str, val: &Value) -> bool {
        match self {
            Value::AExpr(e) => e.try_replace(var, val),
            Value::BExpr(e) => e.try_replace(var, val),
            Value::Enum(_, _, vals) => vals.iter_mut().all(|v| v.try_replace(var, val)),
            Value::Var(name) => {
                if var == name {
                    *self = val.clone();
                }
                true
            }
        }
    }
    pub fn mangle(&self, s: String) -> String {
        s + "#"
            + &match self {
                Value::AExpr(e) => e.to_string(),
                Value::BExpr(e) => e.to_string(),
                Value::Enum(ty, tag, vals) => ty.clone() + "::" + tag + &vals.iter().join("|"),
                Value::Var(name) => name.clone(),
            }
    }
}
impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::AExpr(e) => write!(f, "{e}"),
            Value::BExpr(e) => write!(f, "{e}"),
            Value::Enum(ty, tag, vals) => {
                if vals.is_empty() {
                    write!(f, "{ty}::{tag}")
                } else {
                    write!(f, "{ty}::{tag}({})", vals.iter().join(","))
                }
            }
            Value::Var(name) => write!(f, "{name}"),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum AExpr {
    Var(String),
    Lit(u32),
    Add(Box<AExpr>, Box<AExpr>),
    Sub(Box<AExpr>, Box<AExpr>),
    Mul(Box<AExpr>, Box<AExpr>),
    Div(Box<AExpr>, Box<AExpr>),
}
impl AExpr {
    fn eval(&self, ctx: &Context) -> u32 {
        let n = match self {
            AExpr::Var(x) => panic!("[error] free variable \"{x}\" found in expression: {self}"),
            AExpr::Lit(n) => *n,
            AExpr::Add(l, r) => l.eval(ctx) + r.eval(ctx),
            AExpr::Sub(l, r) => l.eval(ctx) - r.eval(ctx),
            AExpr::Mul(l, r) => l.eval(ctx) * r.eval(ctx),
            AExpr::Div(l, r) => l.eval(ctx) / r.eval(ctx),
        };
        assert!(n >= ctx.bounds().0 && n <= ctx.bounds().1);
        n
    }
    fn try_replace(&mut self, var: &str, val: &Value) -> bool {
        match self {
            AExpr::Var(name) => {
                if let Value::AExpr(e) = val {
                    if var == name {
                        *self = e.clone();
                    }
                    true
                } else {
                    false
                }
            }
            AExpr::Add(l, r) | AExpr::Sub(l, r) | AExpr::Mul(l, r) | AExpr::Div(l, r) => {
                l.try_replace(var, val) && r.try_replace(var, val)
            }
            AExpr::Lit(_) => true,
        }
    }
}
impl Display for AExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AExpr::Var(x) => write!(f, "{}", x),
            AExpr::Lit(n) => write!(f, "{}", n),
            AExpr::Add(l, r) => write!(f, "({} + {})", l, r),
            AExpr::Sub(l, r) => write!(f, "({} - {})", l, r),
            AExpr::Mul(l, r) => write!(f, "({} * {})", l, r),
            AExpr::Div(l, r) => write!(f, "({} / {})", l, r),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum BExpr {
    Lit(bool),
    Var(String),
    Not(Box<BExpr>),
    And(Box<BExpr>, Box<BExpr>),
    Or(Box<BExpr>, Box<BExpr>),
    NumEq(AExpr, AExpr),
    NumNotEq(AExpr, AExpr),
    NumLt(AExpr, AExpr),
    NumGt(AExpr, AExpr),
    NumLtEq(AExpr, AExpr),
    NumGtEq(AExpr, AExpr),
}
impl BExpr {
    pub fn eval(&self, ctx: &Context) -> bool {
        match self {
            BExpr::Lit(true) => true,
            BExpr::Lit(false) => false,
            BExpr::Var(x) => panic!("[error] free variable {x}"),
            BExpr::Not(e) => !e.eval(ctx),
            BExpr::And(l, r) => l.eval(ctx) && r.eval(ctx),
            BExpr::Or(l, r) => l.eval(ctx) || r.eval(ctx),
            BExpr::NumEq(l, r) => l.eval(ctx) == r.eval(ctx),
            BExpr::NumNotEq(l, r) => l.eval(ctx) != r.eval(ctx),
            BExpr::NumLt(l, r) => l.eval(ctx) < r.eval(ctx),
            BExpr::NumGt(l, r) => l.eval(ctx) > r.eval(ctx),
            BExpr::NumLtEq(l, r) => l.eval(ctx) <= r.eval(ctx),
            BExpr::NumGtEq(l, r) => l.eval(ctx) >= r.eval(ctx),
        }
    }
    pub fn try_replace(&mut self, var: &str, val: &Value) -> bool {
        match self {
            BExpr::Lit(_) => true,
            BExpr::Var(name) => {
                if let Value::BExpr(e) = val {
                    if name == var {
                        *self = e.clone();
                    }
                    true
                } else {
                    false
                }
            }
            BExpr::Not(e) => e.try_replace(var, val),
            BExpr::And(l, r) | BExpr::Or(l, r) => {
                l.try_replace(var, val) && r.try_replace(var, val)
            }
            BExpr::NumEq(l, r)
            | BExpr::NumNotEq(l, r)
            | BExpr::NumLt(l, r)
            | BExpr::NumGt(l, r)
            | BExpr::NumLtEq(l, r)
            | BExpr::NumGtEq(l, r) => l.try_replace(var, val) && r.try_replace(var, val),
        }
    }
}
impl Display for BExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BExpr::Lit(val) => write!(f, "{val}"),
            BExpr::Var(x) => write!(f, "{x}"),
            BExpr::Not(e) => write!(f, "!{e}"),
            BExpr::And(l, r) => write!(f, "({l} && {r})"),
            BExpr::Or(l, r) => write!(f, "({l} || {r})"),
            BExpr::NumEq(l, r) => write!(f, "({l} == {r})"),
            BExpr::NumNotEq(l, r) => write!(f, "({l} != {r})"),
            BExpr::NumLt(l, r) => write!(f, "({l} < {r})"),
            BExpr::NumGt(l, r) => write!(f, "({l} > {r})"),
            BExpr::NumLtEq(l, r) => write!(f, "({l} <= {r})"),
            BExpr::NumGtEq(l, r) => write!(f, "({l} >= {r})"),
        }
    }
}
