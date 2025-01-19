use ccs::pretty_print::{print_bisimulation, print_transitions};
use clap::Parser;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[arg(short, long = "source", required = true, help = "Source file")]
    pub source: String,
    #[arg(long = "ccs", action, help = "Evaluate as CCS")]
    pub ccs: bool,
    #[arg(long = "ccsvp", action, help = "Evaluate as CCS-VP")]
    pub ccsvp: bool,
    #[arg(long = "eval-lts", action, help = "Print the LTS")]
    pub eval_lts: bool,
    #[arg(long = "eval-ccs", action, help = "Print the CCS")]
    pub eval_ccs: bool,
    #[arg(long = "eval-bisim", action, help = "Print the bisimilarity")]
    pub eval_bisim: bool,
    #[arg(long = "values", action, help = "Print all the values")]
    pub values: bool,
}

fn main() {
    let args = Cli::parse();
    let content = std::fs::read_to_string(args.source).unwrap();
    if args.ccs && args.ccsvp {
        panic!("Cannot specify both languages");
    }
    if !args.ccs && !args.ccsvp {
        panic!("You need to specify a language");
    }
    if args.ccs {
        let ctx = ccs::context::Context::from(content);
        let lts = ctx.to_lts().flatten();
        if args.eval_lts {
            print_transitions(&lts)
        }
        if args.eval_bisim {
            print_bisimulation(&lts.bisimilarity(&lts))
        }
    } else {
        let ctx = ccs_vp::context::Context::from(content);
        if args.values {
            for v in ctx.values() {
                println!("{v}");
            }
        }
        if args.ccs {
            for c in ctx.to_ccs().constants() {
                println!("{:?}", c);
            }
        }
        if args.eval_lts {
            print_transitions(&ctx.to_ccs().to_lts().flatten())
        }
        if args.eval_bisim {
            let lts = ctx.to_ccs().to_lts().flatten();
            print_bisimulation(&lts.bisimilarity(&lts))
        }
    }
}
