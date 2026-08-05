mod capture;
mod cleanup;
mod cli;
mod clipboard;
mod error;
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
            output.link_found(&sanitized);
            if cli.no_clipboard {
                println!("Clipboard skipped (--no-clipboard).");
                println!();
                println!("Done.");
                println!();
            } else {
                match clipboard::copy(&sanitized) {
                    Ok(()) => output.copied(),
                    Err(err) => {
                        if cli.verbose {
                            eprintln!("clipboard error: {err}");
                        }
                        output.copy_failed(&sanitized);
                    }
                }
            }
            0
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

async fn capture_flow(cli: &Cli, output: &Output) -> anyhow::Result<ExitStatus> {
    output.banner();

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

    output.waiting_game();
    let game = process::wait_for_game(cli.game.as_deref(), cli.game_timeout_duration()).await;
    let (pid, name) = match game {
        Some(game) => game,
        None => return Ok(ExitStatus::GameNotRunning),
    };
    output.game_detected(pid, &name);

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
