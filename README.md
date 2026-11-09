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
5. **Immediately** paste it into your gacha tracker (within ~5 minutes).

> ⚠ The token embedded in the URL is short-lived. The game itself only
> considers it valid for ~290 seconds after it is issued
> (`SummonPoolHistoryModel:setToken` sets `_tokenEndTime = Time.time + 300 - 10`),
> and the server rejects an expired token with HTTP 200 but body
> `{"msg":"没有权限","code":-6}` ("no permission"). If you hit that error,
> exit the Summon Review page / restart the game, reopen Summon Review,
> and run marconi again to grab a fresh link — never reuse an old link
> from the clipboard.
>
> **Skip the clipboard round-trip entirely** with `--json`, which fetches
> the summon history JSON in the same process, in the same second the URL
> is captured (see CLI options below). That is the recommended mode for
> tracker import — the token can no longer expire between capture and
> request.

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
| `--no-clipboard`         | Do not copy the URL to the clipboard.                                    |
| `--json`                 | Fetch the summon history JSON right after capture and print it to stdout (stdout stays clean JSON; status goes to stderr), e.g. `marconi --json > history.json`. |
| `--json-file <path>`     | Like `--json`, but write the JSON to a file instead of stdout.           |
| `--fetch-timeout <secs>` | HTTP timeout for `--json` mode (default 15).                             |
| `-v, --verbose`          | Verbose output.                                                          |

## Testing without the game

Don't want to launch Reverse:1999 just to try it? Generate a fake memory dump
and scan it — the full pipeline (scan → extract → sanitize → clipboard) runs
the same way:

```sh
cargo build --release
./target/release/make-dummy-dump /tmp/dummy.dmp
./target/release/marconi --dump-file /tmp/dummy.dmp
```

Expected result:

```
✔ Link Found

https://game-re-en-service.sl916.com/query/summon?userId=12345&token=123e4567-e89b-12d3-a456-426614174000

Copied to clipboard.
```

The dump is 4 MiB and contains the summon URL as both UTF-8 (straddling a
chunk boundary) and UTF-16LE, so it exercises the chunked scanner too.

## How it works

1. **Process Detector** finds the game process (`Reverse1999.exe` / Steam `3092660`).
2. **Capture Engine (Strategy 1: memory scan)** attaches to the process (`ptrace` + `process_vm_readv`), reads memory regions, searches for the `com/query/summon` string (UTF-8 **and** UTF-16LE), and extracts the full URL around it.
3. **URL Extractor** validates the URL (`/query/summon` + both `userId` and
   `token` parameters — a URL missing either can only fail server-side),
   cleans up `&amp;`, and keeps only the `userId` + `token` parameters in the
   game's own order (the format used by gacha trackers).
4. **Clipboard** copies the URL; if it fails, the URL is still printed.
   With `--json` / `--json-file` instead, the **Fetcher** immediately
   replays that same GET request against the game service (exactly what
   trackers do) and prints/saves the returned JSON — no clipboard, no
   expiry window. Status text goes to stderr so stdout stays pure JSON.
5. **Cleanup Manager** removes the runtime directories (`cache/ temp/ logs/`) and restores any changes on exit (including Ctrl+C / SIGTERM).

All temporary files are created inside the program's folder and cleaned up when done. By default no data is sent anywhere; with `--json` the
only network request is the summon-history GET to the game's own service
(the same request the game itself just made), and nothing — neither the
URL/token nor the JSON — is stored unless you explicitly pass
`--json-file`.

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
  `VirtualQueryEx` + `ReadProcessMemory`) are implemented; the CI workflow
  builds the Windows binary natively on `windows-latest`. Not yet
  runtime-tested on Windows.

## Troubleshooting

### Link found, but the tracker says `{"msg":"没有权限","code":-6}`

That is the server rejecting the token, not a capture failure: the link
itself was fine, but its `token` had already expired by the time the
tracker requested it (the game only keeps a token valid for ~290 s, and
other trackers document the same "no permission" expiry). Fix, in order
of preference:

1. Use `--json` / `--json-file` mode — marconi fetches the JSON itself
   the second the URL is captured, so there is no window to expire in.
2. Or manually: exit the Summon Review page (or restart the game),
   reopen Summon Review in-game, run marconi again, and import the fresh
   link **immediately** (seconds, not minutes). Do not reuse a link
   copied earlier.

## Development

```sh
cargo fmt
cargo test
cargo clippy --all-targets -- -D warnings
cargo build --release
```

CI (`.github/workflows/ci.yml`) runs fmt, clippy, tests, and builds on both
Linux and Windows.

The architecture follows `AGENT.md`: `CLI`, `Process Detector`, `Capture Engine`
(trait + strategy orchestrator), `URL Extractor`, `Clipboard`, `Cleanup Manager`,
`Output`. Upcoming strategies (temporary HTTPS proxy, native hook) are available
as stubs behind the same trait.
