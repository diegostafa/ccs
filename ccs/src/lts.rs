use std::collections::HashSet;

use itertools::Itertools;

use super::process::{Channel, Process};

pub type Transition = (Process, Channel, Process);
pub type Bisimulation<'a, 'b> = HashSet<(&'a Process, &'b Process)>;

#[derive(Clone, Debug, PartialEq)]
pub struct Lts(HashSet<Transition>);
impl Lts {
    pub fn new(ts: HashSet<Transition>) -> Self {
        Self(ts)
    }
    pub fn from_array<const N: usize>(t: [(&Process, &Channel, &Process); N]) -> Self {
        Self(
            t.into_iter()
                .map(|t| (t.0.clone(), t.1.clone(), t.2.clone()))
                .collect(),
        )
    }
    pub fn weaken(self) -> Self {
        let mut transitions = self.0.clone();
        for s in self.nodes() {
            transitions.insert((s.clone(), Channel::Tau, s.clone()));
        }
        for (p, _, q) in self.0.iter().filter(|t| t.1.is_tau()) {
            for before in self.transitions_to(p) {
                transitions.insert((before.0.clone(), before.1.clone(), q.clone()));
            }
            for after in self.transitions_from(q) {
                transitions.insert((p.clone(), after.1.clone(), after.2.clone()));
            }
        }
        Self(transitions)
    }
    pub fn flatten(self) -> Self {
        Self(
            self.0
                .into_iter()
                .map(|t| (t.0.flatten(), t.1, t.2.flatten()))
                .collect(),
        )
    }
    pub fn edges(&self) -> HashSet<&Channel> {
        self.0.iter().map(|t| &t.1).collect()
    }
    pub fn nodes(&self) -> HashSet<&Process> {
        self.0.iter().flat_map(|t| [&t.0, &t.2]).collect()
    }
    pub fn transitions(&self) -> &HashSet<Transition> {
        &self.0
    }
    pub fn transitions_from(&self, p: &Process) -> HashSet<&Transition> {
        self.0.iter().filter(|t| t.0 == *p).collect()
    }
    pub fn transitions_to(&self, p: &Process) -> HashSet<&Transition> {
        self.0.iter().filter(|t| t.2 == *p).collect()
    }
    pub fn bisimilarity<'a, 'b>(&'a self, other: &'b Lts) -> Bisimulation<'a, 'b> {
        let top = self
            .nodes()
            .into_iter()
            .cartesian_product(other.nodes().iter().copied())
            .collect();
        let f = |r: &Bisimulation<'a, 'b>| {
            let is_similar = |this: &Lts, other: &Lts, p: &Process, q: &Process| {
                this.transitions_from(p).iter().all(|pt| {
                    other
                        .transitions_from(q)
                        .iter()
                        .any(|qt| pt.1 == qt.1 && r.contains(&(&pt.2, &qt.2)))
                })
            };
            r.iter()
                .filter(|(p, q)| is_similar(self, other, p, q) && is_similar(other, self, q, p))
                .cloned()
                .collect()
        };
        gfp(f, top)
    }
}

fn gfp<R: PartialEq>(f: impl Fn(&R) -> R, mut curr: R) -> R {
    loop {
        let new = f(&curr);
        if new == curr {
            return curr;
        }
        curr = new;
    }
}
