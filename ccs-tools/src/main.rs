mod cli;
mod renderer;

#[macroquad::main("LtsRenderer")]
async fn main() {
    cli::Cli::parse_args().exec().await;
}
