use std::collections::HashSet;
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
    pub fn to_ccs(
        self,
        ctx: &Context,
        ccs: &mut ContextCcs,
        seen: &mut HashSet<String>,
    ) -> ProcessCcs {
        match self {
            Process::Constant(name, vals) => {
                let vals = vals.iter().map(|v| v.eval(ctx)).collect_vec();
                let encoded = encode_constant(name.to_string(), &vals);
                if seen.contains(&encoded) {
                    return ProcessCcs::constant(encoded);
                }
                let (vars, mut body) = ctx
                    .get_process(&name)
                    .unwrap_or_else(|| panic!("[error] constant {name} not found"))
                    .clone();
                if !vars
                    .iter()
                    .zip(vals.iter())
                    .all(|(var, val)| body.try_replace(var, val))
                {
                    panic!("[error] failed to replace {vars:?} with {vals:?}");
                }
                seen.insert(encoded.clone());
                let body = body.clone().to_ccs(ctx, ccs, seen).flatten();
                ccs.bind_process(encoded.clone(), body);
                seen.remove(&encoded);
                ProcessCcs::constant(encoded)
            }
            Process::Action(ch, p) => match ch {
                Channel::Tau => ProcessCcs::action(ChannelCcs::Tau, p.to_ccs(ctx, ccs, seen)),
                Channel::Recv(name, None) => {
                    ProcessCcs::action(ChannelCcs::Recv(name), p.to_ccs(ctx, ccs, seen))
                }
                Channel::Send(name, None) => {
                    ProcessCcs::action(ChannelCcs::Send(name), p.to_ccs(ctx, ccs, seen))
                }
                Channel::Send(name, Some(e)) => ProcessCcs::action(
                    ChannelCcs::Send(encode_action(name, &e.eval(ctx))),
                    p.to_ccs(ctx, ccs, seen),
                ),
                Channel::Recv(name, Some(var)) => {
                    let mut possibles = vec![];
                    for val in ctx.values() {
                        let mut p = p.clone();
                        if p.try_replace(&var, &val) {
                            possibles.push(ProcessCcs::action(
                                ChannelCcs::Recv(encode_action(name.clone(), &val.eval(ctx))),
                                p.to_ccs(ctx, ccs, seen),
                            ));
                        }
                    }
                    ProcessCcs::sum(possibles)
                }
            },
            Process::Sum(sum) => {
                ProcessCcs::sum(sum.into_iter().map(|p| p.to_ccs(ctx, ccs, seen)).collect())
            }
            Process::Par(p, q) => {
                ProcessCcs::par(p.to_ccs(ctx, ccs, seen), q.to_ccs(ctx, ccs, seen))
            }
            Process::IfThen(b, p) => {
                if b.eval(ctx) {
                    p.to_ccs(ctx, ccs, seen)
                } else {
                    ProcessCcs::nil()
                }
            }
            Process::Restriction(p, chans) => {
                let values = ctx.values();
                let chans = chans
                    .iter()
                    .flat_map(|ch| values.iter().map(|v| encode_action(ch.clone(), v)))
                    .collect();
                ProcessCcs::restriction(p.to_ccs(ctx, ccs, seen), chans)
            }
            Process::Substitution(p, subs) => {
                let values = ctx.values();
                let chans = subs
                    .pairs()
                    .iter()
                    .flat_map(|(new, old)| {
                        values
                            .iter()
                            .map(|v| (encode_action(new.clone(), v), encode_action(old.clone(), v)))
                    })
                    .collect();
                ProcessCcs::substitution(p.to_ccs(ctx, ccs, seen), Substitution::new(chans))
            }
        }
    }
    fn try_replace(&mut self, var: &str, val: &Value) -> bool {
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
                if vals.is_empty() {
                    write!(f, "{k}")
                } else {
                    write!(f, "{k}({})", vals.iter().map(|v| format!("{v}")).join(","))
                }
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

fn encode_action(name: String, v: &Value) -> String {
    name + "#" + &v.to_string()
}
fn encode_constant(name: String, vals: &[Value]) -> String {
    if vals.is_empty() {
        name
    } else {
        name + "#" + &vals.iter().join("#")
    }
}
