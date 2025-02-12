use ccs::utils::run_example;
use ccs_vp::context::Context as ContextVp;
use ccs_vp::utils::print_values;

fn main() {
    let source = include_str!("ccsvp_values.ccsvp");
    let ctx = ContextVp::try_from(source).unwrap();
    run_example(ctx.to_ccs());
    print_values(ctx);
}
