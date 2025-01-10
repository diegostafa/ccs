use ccs::pretty_print::{print_bisimulation, print_relations};
use ccs_vp::ast::Program;
use ccs_vp::context::Context as ContextVp;

fn main() {
    let file = std::fs::read_to_string("ccs-vp/examples/example.ccsvp").unwrap();
    let vp = ContextVp::from(Program::parse(&file).unwrap());
    let ctx_ccs = vp.to_ccs();
    for c in ctx_ccs.constants() {
        println!("{:?}", c);
    }
    let lts = ctx_ccs.to_lts();
    print_relations(&lts);
    println!("bisimulations: ");
    for b in lts.find_bisimulations(&lts) {
        print_bisimulation(&b);
    }

    println!("bisimilarity: ");
    let bimilarity = lts.find_bisimilarity();
    print_bisimulation(&bimilarity);
    assert!(lts.check_bisimulation(&lts, &bimilarity));
}
