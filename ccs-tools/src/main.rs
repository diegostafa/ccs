use ccs::utils::{print_bisimulation, print_ccs, print_transitions};
use clap::Parser;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[arg(short, long = "source", required = true, help = "Source file")]
    pub source: String,
    #[arg(long = "ccs", action, help = "Print the CCS")]
    pub ccs: bool,
    #[arg(long = "bisim", action, help = "Print the bisimilarity")]
    pub bisim: bool,
    #[arg(long = "lts", action, help = "Print the LTS")]
    pub lts: bool,
}

fn main() {
    let args = Cli::parse();
    let source = std::fs::read_to_string(args.source).unwrap();
    let ccs = ccs_vp::context::Context::try_from(source.as_str())
        .map_or_else(
            |_| ccs::context::Context::try_from(source.as_str()),
            |ctx| Ok(ctx.to_ccs()),
        )
        .unwrap();

    if args.ccs {
        print_ccs(&ccs)
    }
    if args.lts {
        print_transitions(&ccs.to_lts().flatten())
    }
    if args.bisim {
        let lts = ccs.to_lts().flatten();
        print_bisimulation(&lts.bisimilarity(&lts))
    }
}
