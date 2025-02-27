use ccs::utils::{print_bisimulation, print_ccs, print_transitions};

use crate::renderer::render_lts;

pub enum Arg {
    Flag(String),
    Value(String, String),
}

pub struct Cli {
    source: String,
    ccs: bool,
    bisim: bool,
    lts: bool,
    render: bool,
}
impl Cli {
    fn default() -> Self {
        Self {
            source: Default::default(),
            ccs: Default::default(),
            bisim: Default::default(),
            lts: Default::default(),
            render: Default::default(),
        }
    }
    pub fn parse_args() -> Self {
        std::env::args().fold(Self::default(), |cli, arg| {
            let arg = match arg.split("=").collect::<Vec<_>>().as_slice() {
                [name] => Arg::Flag(name.trim().to_lowercase()),
                [name, value] => Arg::Value(name.trim().to_lowercase(), value.trim().to_string()),
                _ => panic!("Invalid argument: {arg}"),
            };
            cli.apply_arg(arg)
        })
    }
    fn apply_arg(mut self, arg: Arg) -> Self {
        match arg {
            Arg::Flag(name) => match name.as_str() {
                "ccs" => self.ccs = true,
                "bisim" => self.bisim = true,
                "lts" => self.lts = true,
                "render" => self.render = true,
                _ => {}
            },
            Arg::Value(name, value) => {
                if name.as_str() == "source" {
                    self.source = value
                }
            }
        }

        self
    }
    pub async fn exec(self) {
        if !self.source.ends_with(".ccs") && !self.source.ends_with(".ccsvp") {
            panic!("Invalid source file extension: {}", self.source);
        }
        let source = std::fs::read_to_string(&self.source).unwrap();
        let ccs = ccs_vp::context::Context::try_from(source.as_str())
            .map_or_else(
                |_| ccs::context::Context::try_from(source.as_str()),
                |ctx| Ok(ctx.to_ccs()),
            )
            .unwrap();

        if self.ccs {
            print_ccs(&ccs)
        }
        let lts = ccs.to_lts().flatten();
        if self.lts {
            print_transitions(&lts)
        }
        if self.bisim {
            print_bisimulation(&lts.bisimilarity(&lts))
        }
        if self.render {
            render_lts(&lts).await;
        }
    }
}
