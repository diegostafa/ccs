use ccs::context::Context;

fn main() {
    let file_ccs = std::fs::read_to_string("ccs/examples/example.ccs").unwrap();
    let ccs = Context::from(ccs::ast::Program::parse(&file_ccs).unwrap());
    let lts = ccs.to_lts();
    lts.print_relations();
    println!("bisimulations: ");
    for b in lts.find_bisimulations(&lts) {
        b.pretty_print();
    }
}
