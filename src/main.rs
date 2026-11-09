mod capture;
mod cleanup;
mod cli;
mod clipboard;
mod error;
mod fetcher;
mod output;
mod process;
mod url_extractor;
mod util;

use clap::Parser;
use cleanup::CleanupManager;
use cli::Cli;
use output::Output;
use util::exe_dir;

fn main() {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("failed to build tokio runtime");
    let code = rt.block_on(run());
    std::process::exit(code);
}

enum ExitStatus {
    Found(String),
    GameNotRunning,
    NotFound,
}

async fn run() -> i32 {
    let cli = Cli::parse();
    let output = Output::new(cli.verbose);

    let dirs = match exe_dir::ensure_project_runtime_dirs() {
        Ok(dirs) => dirs,
        Err(err) => {
            eprintln!("error: {err}");
            return 2;
        }
    };
    let _cleanup = CleanupManager::new(dirs);

    let signals = signal_future();
    tokio::pin!(signals);

    #[allow(clippy::ignored_unit_patterns)] // tokio::select! expands to `match ()`
    let result = tokio::select! {
        result = capture_flow(&cli, &output) => result,
        _ = &mut signals => {
            println!("\nInterrupted.");
            return 1;
        }
    };

    match result {
        Ok(ExitStatus::Found(url)) => {
            let sanitized = url_extractor::sanitize_url(&url);
            if cli.wants_json() {
                // stdout is reserved for the JSON payload in this mode
                // (so `marconi --json > history.json` produces a clean
                // file), therefore all status text goes to stderr.
                eprintln!("Link found, fetching summon history...");
                match fetch_json(&cli, &sanitized).await {
                    Ok(code) => code,
                    Err(err) => {
                        eprintln!("error: {err}");
                        1
                    }
                }
            } else {
                Output::link_found(&sanitized);
                if cli.no_clipboard {
                    println!("Clipboard skipped (--no-clipboard).");
                    println!();
                    println!("Done.");
                    println!();
                } else {
                    match clipboard::copy(&sanitized) {
                        Ok(()) => Output::copied(),
                        Err(err) => {
                            if cli.verbose {
                                eprintln!("clipboard error: {err}");
                            }
                            Output::copy_failed();
                        }
                    }
                }
                0
            }
        }
        Ok(ExitStatus::GameNotRunning) => {
            println!("Reverse:1999 is not running.");
            println!("Launch the game, then run this program again.");
            1
        }
        Ok(ExitStatus::NotFound) => {
            println!();
            println!("Summon history URL not found within the timeout.");
            println!("Make sure you opened the Summon Review page in-game.");
            1
        }
        Err(err) => {
            eprintln!("error: {err}");
            2
        }
    }
}

async fn fetch_json(cli: &Cli, url: &str) -> anyhow::Result<i32> {
    let url = url.to_string();
    let timeout = cli.fetch_timeout_duration();
    let body = tokio::task::spawn_blocking(move || fetcher::fetch_summon_history(&url, timeout))
        .await
        .map_err(|join| anyhow::anyhow!("fetch task panicked: {join}"))??;

    let parsed = fetcher::parse_response(&body)?;
    if !parsed.is_success() {
        anyhow::bail!(
            "summon history server rejected the token: code {} ({}) — the token has \
             probably expired, exit Summon Review / restart the game, reopen it, \
             and run marconi again",
            parsed.code,
            parsed.msg
        );
    }

    if let Some(path) = &cli.json_file {
        let value: serde_json::Value = serde_json::from_str(&body)?;
        let pretty = serde_json::to_string_pretty(&value)?;
        std::fs::write(path, format!("{pretty}\n"))?;
        eprintln!(
            "Summon history JSON saved to {} ({} records).",
            path.display(),
            parsed.record_count().unwrap_or(0)
        );
    } else {
        println!("{}", body.trim());
    }
    Ok(0)
}

async fn capture_flow(cli: &Cli, output: &Output) -> anyhow::Result<ExitStatus> {
    // In `--json` mode stdout belongs to the JSON payload, so the human
    // progress banner is printed to stderr instead of being piped into
    // the output file.
    if cli.wants_json() {
        eprintln!("==================================");
        eprintln!();
        eprintln!("Marconi — Reverse:1999 Link Grabber");
        eprintln!();
        eprintln!("==================================");
        eprintln!();
    } else {
        Output::banner();
    }

    let config = capture::CaptureConfig {
        game_pid: None,
        timeout: cli.timeout_duration(),
    };

    if let Some(dump) = &cli.dump_file {
        use capture::CaptureEngine;
        let mut engine = capture::memscan::MemScanEngine::new(Some(dump.clone()));
        return match engine.capture(&config, output).await? {
            Some(url) => Ok(ExitStatus::Found(url)),
            None => Ok(ExitStatus::NotFound),
        };
    }

    if !cli.wants_json() {
        Output::waiting_game();
    }
    let game = process::wait_for_game(cli.game.as_deref(), cli.game_timeout_duration()).await;
    let Some((pid, name)) = game else {
        return Ok(ExitStatus::GameNotRunning);
    };
    if cli.wants_json() {
        eprintln!("Game detected. (pid: {pid}, {name})");
        eprintln!("Waiting for summon history...");
    } else {
        Output::game_detected(pid, &name);
    }

    let config = capture::CaptureConfig {
        game_pid: Some(pid),
        timeout: cli.timeout_duration(),
    };

    match capture::run_strategies(&config, output).await? {
        Some(url) => Ok(ExitStatus::Found(url)),
        None => Ok(ExitStatus::NotFound),
    }
}

#[cfg(unix)]
async fn signal_future() {
    use tokio::signal::unix::{signal, SignalKind};

    let mut terminate = signal(SignalKind::terminate()).expect("failed to install SIGTERM handler");
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {}
        _ = terminate.recv() => {}
    }
}

#[cfg(not(unix))]
async fn signal_future() {
    let _ = tokio::signal::ctrl_c().await;
}
