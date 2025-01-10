use ccs::context::Context;
use ccs::pretty_print::{print_bisimulation, print_relations};

fn main() {
    let file_ccs = std::fs::read_to_string("ccs/examples/example.ccs").unwrap();
    let ccs = Context::from(ccs::ast::Program::parse(&file_ccs).unwrap());
    let lts = ccs.to_lts();
    print_relations(&lts);
    println!("bisimulations: ");
    for b in lts.find_bisimulations(&lts) {
        print_bisimulation(&b);
    }
    println!("bisimilarity: ");
    let bimilarity = lts.find_bisimilarity();
    print_bisimulation(&bimilarity);
}
