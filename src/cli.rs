use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "mock", about = "Mock API server — TUI or background daemon")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,

    /// Path to the SQLite database file
    #[arg(long, default_value = "apimock.db", global = true)]
    pub db: String,

    /// Management port for dashboard/serve mode
    #[arg(long, default_value = "9999", global = true)]
    pub port: u16,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Start the mock server as a background daemon
    Start,
    /// Stop the background mock server
    Stop,
    /// Restart the background mock server (stop + start), reloading all port and API config
    Restart,
    /// Show status of the background mock server
    Status,
    /// Run the mock server in the foreground (ports + web dashboard)
    Serve,
}
