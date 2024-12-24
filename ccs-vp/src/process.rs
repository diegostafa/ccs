use std::fmt::Display;

use ccs::context::Context as ContextCcs;
use ccs::process::{Channel as ChannelCcs, Process as ProcessCcs, Substitution};
use itertools::Itertools;

use super::context::Context;
use super::values::{BExpr, Value};

#[derive(Debug, PartialEq, Clone)]
pub enum Process {
    Constant(String, Vec<Value>),
    Action(Channel, Box<Process>),
    Sum(Vec<Process>),
    Par(Box<Process>, Box<Process>),
    Restriction(Box<Process>, Vec<String>),
    Substitution(Box<Process>, Substitution),
    IfThen(BExpr, Box<Process>),
}
impl Process {
    pub fn try_replace(&mut self, var: &str, val: &Value) -> bool {
        match self {
            Process::Constant(_, vals) => vals.iter_mut().all(|v| v.try_replace(var, val)),
            Process::Sum(vec) => vec.iter_mut().all(|p| p.try_replace(var, val)),
            Process::Action(m, p) => match m {
                Channel::Recv(_, Some(x)) if x == var => true,
                Channel::Send(_, Some(e)) => e.try_replace(var, val) && p.try_replace(var, val),
                _ => p.try_replace(var, val),
            },
            Process::Par(p, q) => p.try_replace(var, val) && q.try_replace(var, val),
            Process::IfThen(b, p) => b.try_replace(var, val) && p.try_replace(var, val),
            Process::Restriction(p, _) => p.try_replace(var, val),
            Process::Substitution(p, _) => p.try_replace(var, val),
        }
    }
    pub fn gen_constants(&self, ctx: &Context, ccs_ctx: &mut ContextCcs) {
        match self {
            Process::Constant(name, vals) => {
                let vals = vals.iter().map(|v| v.eval(ctx)).collect_vec();
                let (vars, mut body) = ctx.get_process(name).unwrap().clone();
                if !vars
                    .iter()
                    .zip(vals.iter())
                    .all(|(var, val)| body.try_replace(var, val))
                {
                    panic!("[error] failed to replace {vars:?} with {vals:?}");
                }

                let name = name.clone() + "#" + &vals.iter().join("#");
                if ccs_ctx.get_process(&name).is_none() {
                    ccs_ctx.bind_process(name, body.clone().to_ccs(ctx));
                    body.gen_constants(ctx, ccs_ctx);
                }
            }
            Process::Action(_, p) => p.gen_constants(ctx, ccs_ctx),
            Process::Sum(sum) => sum.iter().for_each(|p| p.gen_constants(ctx, ccs_ctx)),
            Process::Par(p, q) => {
                p.gen_constants(ctx, ccs_ctx);
                q.gen_constants(ctx, ccs_ctx);
            }
            Process::IfThen(b, p) => {
                if b.eval(ctx) {
                    p.gen_constants(ctx, ccs_ctx)
                }
            }
            Process::Restriction(p, _) => p.gen_constants(ctx, ccs_ctx),
            Process::Substitution(p, _) => p.gen_constants(ctx, ccs_ctx),
        }
    }
    pub fn to_ccs(self, ctx: &Context) -> ProcessCcs {
        match self {
            Process::Constant(name, vals) => {
                ProcessCcs::constant(name + "#" + &vals.into_iter().map(|v| v.eval(ctx)).join("#"))
            }
            Process::Action(ch, p) => match ch {
                Channel::Tau => ProcessCcs::action(ChannelCcs::Tau, p.to_ccs(ctx)),
                Channel::Recv(name, None) => {
                    ProcessCcs::action(ChannelCcs::Recv(name), p.to_ccs(ctx))
                }
                Channel::Send(name, None) => {
                    ProcessCcs::action(ChannelCcs::Send(name), p.to_ccs(ctx))
                }
                Channel::Send(name, Some(e)) => {
                    ProcessCcs::action(ChannelCcs::Send(e.eval(ctx).mangle(name)), p.to_ccs(ctx))
                }
                Channel::Recv(name, Some(var)) => {
                    let mut possibles = vec![];
                    for val in ctx.values() {
                        let mut p = p.clone();
                        if p.try_replace(&var, &val) {
                            possibles.push(ProcessCcs::action(
                                ChannelCcs::Recv(val.mangle(name.clone())),
                                p.to_ccs(ctx),
                            ));
                        }
                    }
                    ProcessCcs::sum(possibles)
                }
            },
            Process::Sum(sum) => ProcessCcs::sum(
                sum.into_iter()
                    .filter_map(|p| match p.to_ccs(ctx) {
                        ProcessCcs::Sum(sum) if sum.is_empty() => None,
                        p => Some(p),
                    })
                    .collect(),
            ),
            Process::Par(p, q) => ProcessCcs::par(p.to_ccs(ctx), q.to_ccs(ctx)),
            Process::IfThen(b, p) => {
                if b.eval(ctx) {
                    p.to_ccs(ctx)
                } else {
                    ProcessCcs::nil()
                }
            }
            Process::Restriction(p, chans) => {
                let values = ctx.values();
                let chans = chans
                    .iter()
                    .flat_map(|ch| values.iter().map(|v| v.mangle(ch.clone())))
                    .collect();
                ProcessCcs::restriction(p.to_ccs(ctx), chans)
            }
            Process::Substitution(p, subs) => {
                let values = ctx.values();
                let chans = subs
                    .pairs()
                    .iter()
                    .flat_map(|(new, old)| {
                        values
                            .iter()
                            .map(|v| (v.mangle(new.clone()), v.mangle(old.clone())))
                    })
                    .collect();
                ProcessCcs::substitution(p.to_ccs(ctx), Substitution::new(chans))
            }
        }
    }

    pub fn nil() -> Self {
        Process::sum(vec![])
    }
    pub fn constant(s: impl Into<String>, vals: Vec<Value>) -> Self {
        Process::Constant(s.into(), vals)
    }
    pub fn action(l: Channel, p: Self) -> Self {
        Process::Action(l, Box::new(p))
    }
    pub fn sum(procs: Vec<Self>) -> Self {
        Process::Sum(procs)
    }
    pub fn par(p: Self, q: Self) -> Self {
        Process::Par(Box::new(p), Box::new(q))
    }
    pub fn restriction(p: Self, chans: Vec<String>) -> Self {
        Process::Restriction(Box::new(p), chans)
    }
    pub fn substitution(p: Self, subs: Substitution) -> Self {
        Process::Substitution(Box::new(p), subs)
    }
    pub fn if_then(b: BExpr, p: Self) -> Self {
        Process::IfThen(b, Box::new(p))
    }
    pub fn if_then_else(b: BExpr, p: Self, q: Self) -> Self {
        Process::sum(vec![
            Process::IfThen(b.clone(), Box::new(p)),
            Process::IfThen(BExpr::Not(Box::new(b)), Box::new(q)),
        ])
    }
}
impl Display for Process {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Process::Constant(k, vals) => {
                write!(f, "{k}({})", vals.iter().map(|v| format!("{v}")).join(","))
            }
            Process::Action(ch, p) => {
                write!(f, "{ch}; {p}")
            }
            Process::Sum(procs) => {
                if procs.is_empty() {
                    write!(f, "NIL")
                } else {
                    write!(f, "({})", procs.iter().join(" + "))
                }
            }
            Process::Par(p, q) => write!(f, "({p} | {q})"),
            Process::Restriction(p, chans) => write!(f, "({p} \\ [{}])", chans.iter().join(", ")),
            Process::IfThen(b, p) => {
                write!(f, "if {b} then {{ {p} }}")
            }
            Process::Substitution(p, subs) => write!(
                f,
                "({p} [{}])",
                subs.pairs()
                    .iter()
                    .map(|(new, old)| format!("{new}/{old}"))
                    .join(", ")
            ),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Channel {
    Send(String, Option<Value>),
    Recv(String, Option<String>),
    Tau,
}
impl Channel {
    pub fn tau() -> Self {
        Channel::Tau
    }
}
impl Display for Channel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Channel::Send(name, Some(val)) => write!(f, "{name}!({val})"),
            Channel::Recv(name, Some(var)) => write!(f, "{name}?({var})"),
            Channel::Send(name, None) => write!(f, "{name}!"),
            Channel::Recv(name, None) => write!(f, "{name}?"),
            Channel::Tau => write!(f, "Tau"),
        }
    }
}
