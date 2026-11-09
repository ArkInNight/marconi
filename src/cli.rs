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

    /// Fetch the summon history JSON immediately after capturing the URL
    /// (while the token is still fresh) and print it to stdout, instead
    /// of only printing/copying the link. Use this to pipe the JSON
    /// straight into a tracker: `marconi --json > history.json`.
    #[arg(long)]
    pub json: bool,

    /// Like `--json`, but write the JSON to this file instead of stdout.
    /// The file contains your summon history, not the token/URL.
    #[arg(long, value_name = "PATH")]
    pub json_file: Option<PathBuf>,

    /// Seconds to wait for the summon history HTTP response in `--json`
    /// mode.
    #[arg(long, default_value_t = 15)]
    pub fetch_timeout: u64,

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

    pub fn fetch_timeout_duration(&self) -> Duration {
        Duration::from_secs(self.fetch_timeout)
    }

    pub fn wants_json(&self) -> bool {
        self.json || self.json_file.is_some()
    }
}
