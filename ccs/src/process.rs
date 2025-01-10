use std::collections::HashSet;
use std::fmt::Display;
use std::hash::Hash;

use itertools::Itertools;

use super::context::Context;
use super::lts::Transition;
use crate::lts::Lts;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Process {
    Constant(String),
    Action(Channel, Box<Process>),
    Sum(Vec<Process>),
    Par(Box<Process>, Box<Process>),
    Substitution(Box<Process>, Substitution),
    Restriction(Box<Process>, Vec<String>),
}
impl Process {
    pub fn is_nil(&self) -> bool {
        match self {
            Process::Sum(sum) => sum.is_empty(),
            _ => false,
        }
    }
    pub fn nil() -> Self {
        Process::Sum(vec![])
    }
    pub fn constant(s: impl Into<String>) -> Self {
        Process::Constant(s.into())
    }
    pub fn action(l: Channel, p: Self) -> Self {
        Process::Action(l, Box::new(p))
    }
    pub fn sum(sum: Vec<Self>) -> Self {
        Process::Sum(sum)
    }
    pub fn par(p: Self, q: Self) -> Self {
        Process::Par(Box::new(p), Box::new(q))
    }
    pub fn substitution(p: Self, subs: Substitution) -> Self {
        Process::Substitution(Box::new(p), subs)
    }
    pub fn restriction(p: Self, chans: Vec<String>) -> Self {
        Process::Restriction(Box::new(p), chans)
    }

    pub fn flatten(self) -> Self {
        match self {
            Process::Constant(_) => self,
            Process::Action(ch, p) => Process::action(ch, p.flatten()),
            Process::Sum(sum) => {
                let sum = sum
                    .into_iter()
                    .map(Process::flatten)
                    .filter(|p| !p.is_nil())
                    .collect_vec();
                if sum.len() == 1 {
                    return sum[0].clone();
                }
                Process::sum(sum)
            }
            Process::Par(p, q) => {
                let p = p.flatten();
                let q = q.flatten();
                if p.is_nil() && q.is_nil() {
                    return Process::nil();
                }
                if p.is_nil() {
                    return q;
                }
                if q.is_nil() {
                    return p;
                }
                Process::par(p, q)
            }
            Process::Substitution(p, subs) => Process::substitution(p.flatten(), subs),
            Process::Restriction(p, chans) => Process::restriction(p.flatten(), chans),
        }
    }

    pub fn unfold(self, ctx: &Context) -> Self {
        let mut seen = HashSet::new();
        if let Some(name) = ctx.name_of(&self) {
            seen.insert(name.to_string());
        }
        self.unfold_rec(ctx, &mut seen)
    }
    fn unfold_rec(self, ctx: &Context, seen: &mut HashSet<String>) -> Process {
        match self {
            Process::Constant(name) => {
                if seen.contains(&name) {
                    return Process::Constant(name);
                }
                seen.insert(name.clone());
                let p = ctx
                    .get_process(&name)
                    .unwrap()
                    .clone()
                    .unfold_rec(ctx, seen);
                seen.remove(&name);
                p
            }
            Process::Action(ch, p) => Process::action(ch, p.unfold_rec(ctx, seen)),
            Process::Sum(sum) => {
                Process::sum(sum.into_iter().map(|p| p.unfold_rec(ctx, seen)).collect())
            }
            Process::Par(p, q) => Process::par(p.unfold_rec(ctx, seen), q.unfold_rec(ctx, seen)),
            Process::Substitution(p, subs) => Process::substitution(p.unfold_rec(ctx, seen), subs),
            Process::Restriction(p, chans) => Process::restriction(p.unfold_rec(ctx, seen), chans),
        }
    }

    pub fn derive_lts(self, ctx: &Context) -> Lts {
        let unfolded = self.unfold(ctx);
        let mut transitions = unfolded.derive();
        let mut len = 0;
        while transitions.len() != len {
            len = transitions.len();
            for t in transitions.clone() {
                transitions.extend(t.2.derive());
            }
        }
        Lts::new(
            transitions
                .into_iter()
                .map(|t| (t.0.flatten(), t.1, t.2.flatten()))
                .collect(),
        )
    }

    fn derive(&self) -> HashSet<Transition> {
        match self {
            Process::Constant(_) => Default::default(),
            Process::Action(a, p) => [(self.clone(), a.clone(), *p.clone())]
                .into_iter()
                .collect(),

            Process::Restriction(p, chans) => p
                .derive()
                .into_iter()
                .filter(|t| !chans.contains(&t.1.name().to_string()))
                .map(|t| (self.clone(), t.1, Process::restriction(t.2, chans.clone())))
                .collect(),

            Process::Substitution(p, subs) => p
                .derive()
                .into_iter()
                .map(|t| {
                    (
                        self.clone(),
                        subs.replace_channel(t.1),
                        Process::substitution(t.2, subs.clone()),
                    )
                })
                .collect(),

            Process::Sum(sum) => sum
                .iter()
                .flat_map(|p| p.derive())
                .map(|t| (self.clone(), t.1, t.2))
                .collect(),

            Process::Par(p, q) => {
                let mut ts = HashSet::new();
                let mut ptrans = p.derive();
                let mut qtrans = q.derive();

                for pt in ptrans.clone() {
                    for qt in qtrans.clone() {
                        if pt.1.is_synched_with(&qt.1) {
                            ts.insert((
                                self.clone(),
                                Channel::tau(),
                                Process::par(pt.2.clone(), qt.2.clone()),
                            ));
                            ptrans.remove(&pt);
                            qtrans.remove(&qt);
                        }
                    }
                }
                let ptrans = ptrans
                    .into_iter()
                    .map(|t| (self.clone(), t.1, Process::par(t.2, *q.clone())));

                let qtrans = qtrans
                    .into_iter()
                    .map(|t| (self.clone(), t.1, Process::par(*p.clone(), t.2)));

                ts.extend(ptrans);
                ts.extend(qtrans);
                ts
            }
        }
    }
}
impl Display for Process {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Process::Constant(k) => write!(f, "{}()", k),
            Process::Action(ch, p) => write!(f, "{}.{}", ch, p),
            Process::Sum(procs) => {
                if procs.is_empty() {
                    write!(f, "NIL")
                } else {
                    write!(f, "({})", procs.iter().join(" + "))
                }
            }
            Process::Par(p, q) => write!(f, "({} | {})", p, q),
            Process::Restriction(p, chans) => {
                write!(f, "({} \\ {{ {} }})", p, chans.iter().join(", "))
            }
            Process::Substitution(p, subs) => write!(
                f,
                "({p}[{}])",
                subs.pairs()
                    .iter()
                    .map(|(new, old)| format!("{new}/{old}"))
                    .join(", ")
            ),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Substitution(Vec<(String, String)>);
impl Substitution {
    pub fn new(subs: Vec<(String, String)>) -> Self {
        assert!(subs.iter().all(|(new, old)| new != "tau" && old != "tau"));
        Self(subs)
    }
    pub fn pairs(&self) -> &[(String, String)] {
        &self.0
    }
    pub fn replace(&self, name: String) -> String {
        self.0
            .iter()
            .find_map(|(new, old)| (*old == name).then(|| new.clone()))
            .unwrap_or(name)
    }
    pub fn replace_channel(&self, ch: Channel) -> Channel {
        match ch {
            Channel::Send(name) => Channel::Send(self.replace(name)),
            Channel::Recv(name) => Channel::Recv(self.replace(name)),
            Channel::Tau => Channel::Tau,
        }
    }
    pub fn replace_process(&self, p: Process) -> Process {
        match p {
            Process::Constant(_) => p,
            Process::Action(ch, p) => {
                Process::action(self.replace_channel(ch), self.replace_process(*p))
            }
            Process::Sum(sum) => {
                Process::Sum(sum.into_iter().map(|p| self.replace_process(p)).collect())
            }
            Process::Par(p, q) => Process::Par(
                Box::new(self.replace_process(*p)),
                Box::new(self.replace_process(*q)),
            ),
            Process::Substitution(p, f) => {
                Process::Substitution(Box::new(self.replace_process(*p)), f)
            }
            Process::Restriction(p, chans) => {
                Process::Restriction(Box::new(self.replace_process(*p)), chans)
            }
        }
    }
    pub fn replace_transition(&self, t: Transition) -> Transition {
        (
            self.replace_process(t.0),
            self.replace_channel(t.1),
            self.replace_process(t.2),
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Channel {
    Send(String),
    Recv(String),
    Tau,
}
impl Channel {
    pub fn name(&self) -> &str {
        match self {
            Channel::Send(s) | Channel::Recv(s) => s,
            Channel::Tau => "tau",
        }
    }

    pub fn send(s: &str) -> Channel {
        Channel::Send(s.into())
    }
    pub fn recv(s: &str) -> Channel {
        Channel::Recv(s.into())
    }
    pub fn tau() -> Channel {
        Channel::Tau
    }
    pub fn is_tau(&self) -> bool {
        self == &Channel::Tau
    }
    pub fn is_synched_with(&self, other: &Channel) -> bool {
        match (self, other) {
            (Channel::Send(a), Channel::Recv(b)) | (Channel::Recv(a), Channel::Send(b)) => *a == *b,
            (Channel::Tau, Channel::Tau) => true,
            _ => false,
        }
    }
}

impl Display for Channel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Channel::Send(s) => write!(f, "{}!", s),
            Channel::Recv(s) => write!(f, "{}?", s),
            Channel::Tau => write!(f, "Tau"),
        }
    }
}
