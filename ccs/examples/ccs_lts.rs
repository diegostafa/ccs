use ccs::context::Context;
use ccs::pretty_print::print_bisimulation;

fn main() {
    let file = std::fs::read_to_string("ccs/examples/example.ccs").unwrap();
    let ccs = Context::from(file);
    let lts = ccs.to_lts();
    println!("bisimilarity: ");
    let bimilarity = lts.bisimilarity(&lts);
    print_bisimulation(&bimilarity);
}
