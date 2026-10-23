//! `sim-tools`: validators, text views, record and replay. The agent's eyes.
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "sim-tools",
    about = "Validators, text views, record and replay"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Print the toolchain and crate versions this binary was built with.
    Version,
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Command::Version => println!("sim-tools {}", env!("CARGO_PKG_VERSION")),
    }
}
