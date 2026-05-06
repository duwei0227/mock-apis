use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "mock", about = "Mock API server — TUI, web dashboard, or background daemon")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,

    /// Path to the SQLite database file
    #[arg(long, default_value = "mock-apis.db", global = true)]
    pub db: String,

    /// Management port for dashboard/serve mode
    #[arg(long, default_value = "9999", global = true)]
    pub port: u16,

    /// Launch the web dashboard instead of the TUI (no subcommand only)
    #[arg(long)]
    pub dashboard: bool,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Start the mock server as a background daemon
    Start,
    /// Stop the background mock server
    Stop,
    /// Show status of the background mock server
    Status,
    /// Run the mock server in the foreground (ports + web dashboard)
    Serve,
}
