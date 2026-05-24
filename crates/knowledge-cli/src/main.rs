use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "knowledge-cli")]
#[command(about = "Local knowledge system tooling for aitoolbox")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Query,
    Init,
    Capture,
}

fn main() -> Result<()> {
    tracing_subscriber::fmt().without_time().init();
    let _ = Cli::parse();
    Ok(())
}
