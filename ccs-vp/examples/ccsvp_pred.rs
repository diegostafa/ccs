use ccs::utils::run_example;
use ccs_vp::context::Context;

fn main() {
    let source = include_str!("ccsvp_pred.ccsvp");
    let ctx = Context::try_from(source).unwrap();
    run_example(ctx.to_ccs());
}
