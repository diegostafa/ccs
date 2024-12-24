use std::collections::HashSet;

use cli_tables::Table;

use super::process::{Channel, Process};

pub type Transition = (Process, Channel, Process);

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
        for s in self.get_states() {
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
    pub fn get_actions(&self) -> HashSet<&Channel> {
        self.0.iter().map(|t| &t.1).collect()
    }
    pub fn get_states(&self) -> HashSet<&Process> {
        self.0.iter().flat_map(|t| [&t.0, &t.2]).collect()
    }
    pub fn get_transitions(&self) -> HashSet<Transition> {
        self.0.clone()
    }
    pub fn get_transitions_from(&self, p: &Process) -> HashSet<&Transition> {
        self.0.iter().filter(|t| t.0 == *p).collect()
    }
    pub fn get_transitions_to(&self, p: &Process) -> HashSet<&Transition> {
        self.0.iter().filter(|t| t.2 == *p).collect()
    }
    pub fn print_relations(&self) {
        draw_table(
            vec!["State", "Action", "Next state"],
            self.0
                .iter()
                .map(|(s1, a, s2)| vec![format!("{}", s1), format!("{}", a), format!("{}", s2)])
                .collect(),
        );
    }
    pub fn print_states(&self) {
        draw_table(
            vec!["State"],
            self.get_states()
                .iter()
                .map(|s| vec![format!("{:?}", s)])
                .collect(),
        );
    }
    pub fn print_actions(&self) {
        draw_table(
            vec!["Action"],
            self.get_actions()
                .iter()
                .map(|s| vec![format!("{}", s)])
                .collect(),
        );
    }

    pub fn check_bisimulation<'a, 'b>(
        &'a self,
        other: &'b Self,
        bisim: &Bisimulation<'a, 'b>,
    ) -> bool {
        let check = |this: &Self, other: &Self, bisim: &Bisimulation<'a, 'b>| {
            for (p, q) in &bisim.0 {
                let ptrans = this.get_transitions_from(p);
                let qtrans = other.get_transitions_from(q);
                for pt in ptrans {
                    if !qtrans
                        .iter()
                        .any(|qt| qt.1 == pt.1 && bisim.0.contains(&(&pt.2, &qt.2)))
                    {
                        return false;
                    }
                }
            }
            true
        };
        check(self, other, bisim) && check(other, self, bisim)
    }

    pub fn check_weak_bisimulation<'a, 'b>(
        &'a self,
        other: &'b Self,
        bisim: &Bisimulation<'a, 'b>,
    ) -> bool {
        let self_weak = self.clone().weaken();
        let other_weak = other.clone().weaken();
        self.check_bisimulation(&other_weak, bisim) && other.check_bisimulation(&self_weak, bisim)
    }

    pub fn find_bisimulations<'a, 'b>(&'a self, other: &'b Self) -> Vec<Bisimulation<'a, 'b>> {
        self.get_states()
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
            .get_states()
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
        self.find_bisimulations_between_rec(other, p, q, Bisimulation::default(), HashSet::new())
            .into_iter()
            .filter(|b| self.check_bisimulation(other, b))
            .collect()
    }

    fn find_bisimulations_between_rec<'a, 'b>(
        &'a self,
        other: &'b Self,
        p: &'a Process,
        q: &'b Process,
        bisim: Bisimulation<'a, 'b>,
        checked: HashSet<&'a Transition>,
    ) -> Vec<Bisimulation<'a, 'b>> {
        let mut bisims = vec![];
        let mut ptrans = self.get_transitions_from(p);
        ptrans = ptrans.difference(&checked).cloned().collect();

        if ptrans.is_empty() {
            if !bisim.0.is_empty() {
                bisims.push(bisim.clone());
            }
            return bisims;
        }

        let qtrans = other.get_transitions_from(q);
        for pt in ptrans {
            for qt in &qtrans {
                if pt.1 == qt.1 {
                    let mut checked = checked.clone();
                    checked.insert(pt);
                    let new = self.find_bisimulations_between_rec(
                        other,
                        &pt.2,
                        &qt.2,
                        bisim.clone().append((&pt.0, &qt.0)),
                        checked,
                    );
                    bisims.extend(new);
                }
            }
        }
        bisims
    }
}

#[derive(Debug, Default, Clone, Eq)]
pub struct Bisimulation<'a, 'b>(HashSet<(&'a Process, &'b Process)>);
impl<'a, 'b> Bisimulation<'a, 'b> {
    pub fn insert(&mut self, p: (&'a Process, &'b Process)) {
        self.0.insert(p);
    }
    pub fn append(mut self, p: (&'a Process, &'b Process)) -> Self {
        self.0.insert(p);
        self
    }
    pub fn pretty_print(&self) {
        assert!(!self.0.is_empty());
        draw_table(
            vec!["State", "", "State"],
            self.0
                .iter()
                .map(|(s1, s2)| vec![format!("{:?}", s1), format!("~"), format!("{:?}", s2)])
                .collect(),
        );
    }
}
impl PartialEq for Bisimulation<'_, '_> {
    fn eq(&self, other: &Self) -> bool {
        let l = self
            .0
            .iter()
            .all(|(p1, p2)| other.0.contains(&(p1, p2)) || other.0.contains(&(p2, p1)));
        let r = other
            .0
            .iter()
            .all(|(p1, p2)| self.0.contains(&(p1, p2)) || self.0.contains(&(p2, p1)));
        l && r
    }
}
pub fn draw_table(headers: Vec<&str>, rows: Vec<Vec<String>>) {
    let mut data: Vec<Vec<&str>> = vec![];
    data.push(headers);
    for row in &rows {
        data.push(row.iter().map(|s| s.as_str()).collect());
    }
    let mut table = Table::new();
    table.set(&data).unwrap();
    println!("{}", table.to_string());
}
