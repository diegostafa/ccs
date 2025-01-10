use cli_tables::Table;

use crate::lts::{Bisimulation, Lts};

pub fn draw_table(headers: Vec<&str>, rows: Vec<Vec<String>>) {
    assert!(!headers.is_empty());
    assert!(!rows.is_empty());
    let mut data = vec![];
    data.push(headers);
    for row in &rows {
        data.push(row.iter().map(|s| s.as_str()).collect());
    }
    let mut table = Table::new();
    table.set(&data).unwrap();
    println!("{}", table.to_string());
}

pub fn print_bisimulation(b: &Bisimulation) {
    draw_table(
        vec!["State", "", "State"],
        b.iter()
            .map(|(s1, s2)| vec![format!("{s1}"), format!("~"), format!("{s2}")])
            .collect(),
    );
}

pub fn print_relations(lts: &Lts) {
    draw_table(
        vec!["State", "Action", "Next state"],
        lts.get_transitions()
            .iter()
            .map(|(s1, a, s2)| vec![format!("{s1}"), format!("{a}"), format!("{s2}")])
            .collect(),
    );
}
pub fn print_states(lts: &Lts) {
    draw_table(
        vec!["State"],
        lts.get_nodes()
            .iter()
            .map(|s| vec![format!("{s}")])
            .collect(),
    );
}
pub fn print_actions(lts: &Lts) {
    draw_table(
        vec!["Action"],
        lts.get_edges()
            .iter()
            .map(|a| vec![format!("{a}")])
            .collect(),
    );
}
