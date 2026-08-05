# Marconi — Reverse:1999 Link Grabber

A cross-platform CLI tool that grabs the **Summon History** URL (`/query/summon`) from Reverse:1999 automatically — no Fiddler, no permanent system changes.

Named after **Guglielmo Marconi**, the inventor of radio: just as Marconi caught signals from the air, this tool catches the summon URL from the game.

## Usage

```sh
./marconi
```

1. Run the program.
2. Launch Reverse:1999 (Windows/Steam Proton).
3. Open the **Summon Review** / Summon History page in-game.
4. The URL is detected, copied to the clipboard, and the program exits.

Example output:

```
==================================

Reverse1999 Link Grabber

==================================

Waiting for Reverse:1999...

Game detected. (pid: 30680, Reverse1999.exe)

Waiting for summon history...

Open the Summon Review page in-game when ready.

✔ Link Found

https://game-re-en-service.sl916.com/query/summon?userId=...

Copied to clipboard.

Done.
```

## CLI options

| Option                   | Description                                                              |
| ------------------------ | ------------------------------------------------------------------------ |
| `--game <pattern>`       | Override the game process search pattern (substring, case-insensitive).  |
| `--timeout <seconds>`    | How long to wait for the URL after the game is detected (default 300).   |
| `--game-timeout <secs>`  | How long to wait for the game to launch (0 = wait forever).              |
| `--dump-file <path>`     | Scan a memory dump file instead of a live process (debug/testing).       |
| `--no-clipboard`         | Do not copy to the clipboard.                                            |
| `-v, --verbose`          | Verbose output.                                                          |

## How it works

1. **Process Detector** finds the game process (`Reverse1999.exe` / Steam `3092660`).
2. **Capture Engine (Strategy 1: memory scan)** attaches to the process (`ptrace` + `process_vm_readv`), reads memory regions, searches for the `com/query/summon` string (UTF-8 **and** UTF-16LE), and extracts the full URL around it.
3. **URL Extractor** validates the URL (`/query/summon` + parameters), cleans up `&amp;`, and keeps only the `userId` + `token` parameters (the format used by gacha trackers).
4. **Clipboard** copies the URL; if it fails, the URL is still printed.
5. **Cleanup Manager** removes the runtime directories (`cache/ temp/ logs/`) and restores any changes on exit (including Ctrl+C / SIGTERM).

All temporary files are created inside the program's folder and cleaned up when done. No data is sent anywhere.

## Linux / Proton notes

- Reading another process's memory requires `ptrace` access. Distributions with
  `kernel.yama.ptrace_scope=1` (Arch/Fedora/Ubuntu default) block attaching to
  processes that are not children of this program. Temporary workaround:

  ```sh
  sudo sysctl -w kernel.yama.ptrace_scope=0
  ```

  (applies until reboot; alternatively run the tool with `sudo`). If still
  blocked, the program shows an automatic hint.

## Platform status

- **Linux (Proton)** — active (Strategy 1 memory scan).
- **Windows** — process detection (Toolhelp32) & memory read (`OpenProcess` +
  `VirtualQueryEx` + `ReadProcessMemory`) are implemented and pass the
  compile-check for the `x86_64-pc-windows-gnu` target; not yet runtime-tested
  on Windows.

## Development

```sh
cargo build --release
cargo test
cargo clippy --all-targets
```

The architecture follows `AGENT.md`: `CLI`, `Process Detector`, `Capture Engine`
(trait + strategy orchestrator), `URL Extractor`, `Clipboard`, `Cleanup Manager`,
`Output`. Upcoming strategies (temporary HTTPS proxy, native hook) are available
as stubs behind the same trait.
