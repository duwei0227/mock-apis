use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "mock", about = "Mock API server — TUI or web dashboard")]
pub struct Cli {
    /// Launch the web dashboard instead of the TUI
    #[arg(long)]
    pub dashboard: bool,

    /// Management port for dashboard mode (default: 9999)
    #[arg(long, default_value = "9999")]
    pub port: u16,

    /// Path to the SQLite database file
    #[arg(long, default_value = "mock-apis.db")]
    pub db: String,
}
