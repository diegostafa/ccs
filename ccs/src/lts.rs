use std::collections::HashSet;

use itertools::Itertools;

use super::process::{Channel, Process};

pub type Transition = (Process, Channel, Process);

// for (p, q) in r, if p->p' then q->q' and (p', q') and viceversa
pub type Bisimulation<'a, 'b> = HashSet<(&'a Process, &'b Process)>;

// for (p, q) in r, if p->p' then q->q' and (p', q')
pub type Simulation<'a, 'b> = HashSet<(&'a Process, &'b Process)>;

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
        for s in self.get_nodes() {
            transitions.insert((s.clone(), Channel::Tau, s.clone()));
        }
        for (p, _, q) in self.0.iter().filter(|t| t.1.is_tau()) {
            for before in self.get_transitions_to(p) {
                transitions.insert((before.0.clone(), before.1.clone(), q.clone()));
            }
            for after in self.get_transitions_from(q) {
                transitions.insert((p.clone(), after.1.clone(), after.2.clone()));
            }
        }
        Self(transitions)
    }
    pub fn get_edges(&self) -> HashSet<&Channel> {
        self.0.iter().map(|t| &t.1).collect()
    }
    pub fn get_nodes(&self) -> HashSet<&Process> {
        self.0.iter().flat_map(|t| [&t.0, &t.2]).collect()
    }
    pub fn get_transitions(&self) -> &HashSet<Transition> {
        &self.0
    }
    pub fn get_transitions_from(&self, p: &Process) -> HashSet<&Transition> {
        self.0.iter().filter(|t| t.0 == *p).collect()
    }
    pub fn get_transitions_to(&self, p: &Process) -> HashSet<&Transition> {
        self.0.iter().filter(|t| t.2 == *p).collect()
    }

    pub fn check_bisimulation<'a, 'b>(&'a self, other: &'b Self, r: &Bisimulation<'a, 'b>) -> bool {
        let check = |this: &Self, other: &Self, r: &Simulation<'a, 'b>| {
            for (p, q) in r {
                let ptrans = this.get_transitions_from(p);
                let qtrans = other.get_transitions_from(q);
                for pt in ptrans {
                    if !qtrans
                        .iter()
                        .any(|qt| qt.1 == pt.1 && r.contains(&(&pt.2, &qt.2)))
                    {
                        return false;
                    }
                }
            }
            true
        };
        check(self, other, r) && check(other, self, r)
    }

    pub fn check_weak_bisimulation<'a, 'b>(
        &'a self,
        other: &'b Self,
        r: &Bisimulation<'a, 'b>,
    ) -> bool {
        let self_weak = self.clone().weaken();
        let other_weak = other.clone().weaken();
        self.check_bisimulation(&other_weak, r) && other.check_bisimulation(&self_weak, r)
    }

    pub fn find_bisimulations<'a, 'b>(&'a self, other: &'b Self) -> Vec<Bisimulation<'a, 'b>> {
        self.get_nodes()
            .iter()
            .flat_map(|p| self.find_bisimulation_for(other, p))
            .collect()
    }

    pub fn find_bisimulation_for<'a, 'b>(
        &'a self,
        other: &'b Self,
        p: &'a Process,
    ) -> Vec<Bisimulation<'a, 'b>> {
        other
            .get_nodes()
            .iter()
            .flat_map(|q| self.find_bisimulations_between(other, p, q))
            .collect()
    }

    pub fn find_bisimulations_between<'a, 'b>(
        &'a self,
        other: &'b Self,
        p: &'a Process,
        q: &'b Process,
    ) -> Vec<Bisimulation<'a, 'b>> {
        assert!(self.get_nodes().contains(p));
        assert!(other.get_nodes().contains(q));

        let lhs = self.find_simulations(other, p, q, &mut HashSet::new(), &mut HashSet::new());
        let rhs = other.find_simulations(self, q, p, &mut HashSet::new(), &mut HashSet::new());
        let rhs = rhs
            .into_iter()
            .map(|r| r.iter().map(|(p, q)| (*q, *p)).collect())
            .collect_vec();

        let mut bisims = vec![];
        for ri in &lhs {
            for rj in &rhs {
                if !bisims.contains(ri) && ri == rj {
                    bisims.push(ri.clone());
                }
            }
        }
        bisims
    }

    fn find_simulations<'a, 'b>(
        &'a self,
        other: &'b Self,
        p: &'a Process,
        q: &'b Process,
        r: &mut Simulation<'a, 'b>,
        seen: &mut HashSet<&'a Transition>,
    ) -> Vec<Simulation<'a, 'b>> {
        let ptrans: HashSet<_> = self
            .get_transitions_from(p)
            .difference(seen)
            .cloned()
            .collect();

        if ptrans.is_empty() {
            if !r.is_empty() {
                r.insert((p, q));
                return vec![r.clone()];
            }
            return vec![];
        }

        let mut bisims = vec![];
        let qtrans = other.get_transitions_from(q);
        for pt @ (p1, pa, p2) in &ptrans {
            for (q1, qa, q2) in &qtrans {
                if pa == qa {
                    r.insert((p1, q1));
                    seen.insert(pt);
                    bisims.extend(self.find_simulations(other, p2, q2, r, seen));
                    seen.remove(pt);
                    r.remove(&(p1, q1));
                }
            }
        }
        bisims
    }

    pub fn find_bisimilarity(&self) -> Bisimulation {
        let f = |r: &mut Bisimulation| {
            let mut to_remove = vec![];
            for pair @ (p, q) in r.iter() {
                let ptrans = self.get_transitions_from(p);
                let qtrans = self.get_transitions_from(q);
                for pt in &ptrans {
                    if !qtrans
                        .iter()
                        .any(|qt| qt.1 == pt.1 && r.contains(&(&pt.2, &qt.2)))
                    {
                        to_remove.push(*pair);
                    }
                }
            }
            if to_remove.is_empty() {
                return false;
            }
            for p in to_remove {
                r.remove(&p);
            }
            true
        };

        let nodes = self.get_nodes();
        let top = nodes
            .iter()
            .cloned()
            .cartesian_product(nodes.iter().cloned())
            .collect();
        fix(f, top)
    }
}

fn fix<T>(f: impl Fn(&mut T) -> bool, mut curr: T) -> T {
    while f(&mut curr) {}
    curr
}
