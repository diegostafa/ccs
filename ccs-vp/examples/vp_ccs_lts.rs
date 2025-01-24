use ccs::pretty_print::{print_bisimulation, print_transitions};
use ccs_vp::context::Context as ContextVp;

fn main() {
    let file = std::fs::read_to_string("ccs-vp/examples/example.ccsvp").unwrap();
    let vp = ContextVp::from(file);
    let ctx_ccs = vp.to_ccs();
    for c in ctx_ccs.constants() {
        println!("let {} = {}", c.0, c.1);
    }
    let lts = ctx_ccs.to_lts().flatten();
    print_transitions(&lts);
    println!("bisimilarity: ");
    let bimilarity = lts.bisimilarity(&lts);
    print_bisimulation(&bimilarity);
}
