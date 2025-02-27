use cli_tables::Table;

use crate::context::Context;
use crate::lts::{Bisimulation, Lts};
use crate::process::Process;

pub fn print_bisimulation(b: &Bisimulation) {
    println!("Bisimulation:");
    draw_table(
        vec!["State", "", "State"],
        b.iter()
            .map(|(s1, s2)| {
                if s1 == s2 {
                    vec![format!("{s1}"), format!("="), format!("{s2}")]
                } else {
                    vec![format!("{s1}"), format!("~"), format!("{s2}")]
                }
            })
            .collect(),
    );
}
pub fn print_transitions(lts: &Lts) {
    println!("Transitions:");
    draw_table(
        vec!["State", "Action", "Next state"],
        lts.transitions()
            .iter()
            .map(|(s1, a, s2)| vec![format!("{s1}"), format!("{a}"), format!("{s2}")])
            .collect(),
    );
}
pub fn print_states(lts: &Lts) {
    println!("States:");
    draw_table(
        vec!["State"],
        lts.nodes().iter().map(|s| vec![format!("{s}")]).collect(),
    );
}
pub fn print_actions(lts: &Lts) {
    println!("Actions:");
    draw_table(
        vec!["Action"],
        lts.edges().iter().map(|a| vec![format!("{a}")]).collect(),
    );
}
pub fn print_ccs(ccs: &Context) {
    println!("CCS program:");
    for (name, body) in ccs.constants() {
        println!("fn {name} {{  {body} }}");
    }
}
pub fn print_stats(lts: &Lts) {
    println!("Stats:");
    println!("States: {}", lts.nodes().len());
    println!("Actions: {}", lts.edges().len());
    println!("Transitions: {}", lts.transitions().len());
}
fn draw_table(headers: Vec<&str>, rows: Vec<Vec<String>>) {
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

pub fn run_example(ctx: Context) {
    print_ccs(&ctx);
    println!();
    let lts = Process::constant("main").derive_lts(&ctx).flatten();
    print_transitions(&lts);
    println!();
    print_bisimulation(&lts.bisimilarity(&lts));
    println!();
    print_stats(&lts);
}
