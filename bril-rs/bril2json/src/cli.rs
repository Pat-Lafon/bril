use clap::Parser;

#[derive(Parser)]
#[command(about, version, author)] // keeps the cli synced with Cargo.toml
pub struct Cli {
    /// Flag for whether position information should be included
    #[arg(short, action)]
    pub position: bool,
}
