use ccs::context::Context;
use ccs::utils::run_example;

fn main() {
    let source = include_str!("ccs_circular.ccs");
    let ctx = Context::try_from(source).unwrap();
    run_example(ctx);
}
