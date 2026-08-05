use clap::Parser;
use std::path::PathBuf;
use std::time::Duration;

#[derive(Parser)]
#[command(
    name = "marconi",
    version,
    about = "Grab the Reverse:1999 Summon History URL without Fiddler"
)]
pub struct Cli {
    /// Path to a saved memory dump to scan instead of a live process.
    #[arg(long)]
    pub dump_file: Option<PathBuf>,

    /// Seconds to wait for the summon URL after the game is detected.
    #[arg(long, default_value_t = 300)]
    pub timeout: u64,

    /// Seconds to wait for the game to start (0 = wait forever).
    #[arg(long, default_value_t = 0)]
    pub game_timeout: u64,

    /// Do not copy the URL to the clipboard.
    #[arg(long)]
    pub no_clipboard: bool,

    /// Verbose output.
    #[arg(short, long)]
    pub verbose: bool,

    /// Override the game process name pattern (case-insensitive substring).
    #[arg(long)]
    pub game: Option<String>,
}

impl Cli {
    pub fn timeout_duration(&self) -> Duration {
        Duration::from_secs(self.timeout)
    }

    pub fn game_timeout_duration(&self) -> Option<Duration> {
        match self.game_timeout {
            0 => None,
            secs => Some(Duration::from_secs(secs)),
        }
    }
}
