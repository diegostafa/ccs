use ccs::context::Context;
use ccs::pretty_print::{print_bisimulation, print_transitions};

fn main() {
    let file = std::fs::read_to_string("ccs/examples/example.ccs").unwrap();
    let ccs = Context::from(file);
    let lts = ccs.to_lts();
    print_transitions(&lts);
    println!("bisimilarity: ");
    let bimilarity = lts.bisimilarity(&lts);
    print_bisimulation(&bimilarity);
}
