# Changelog

All notable changes to the Marconi project are recorded in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

## [v0.5] - 2026-11-14

### Added

- `--json` / `--json-file <path>` mode: after capturing the summon URL,
  marconi immediately replays the same HTTPS GET the game itself made
  (via the new `fetcher` module, `ureq` + rustls) and prints/saves the
  returned summon history JSON, instead of only handing the link to a
  tracker. This removes the clipboard round-trip that lost the race
  against the token's ~290 s lifetime. In `--json` mode stdout carries
  only JSON and all status text goes to stderr, so
  `marconi --json > history.json` produces a clean file. The server's
  `code` is checked: `-6` ("没有权限") and empty/non-JSON responses
  produce an actionable error instead of a bogus file.
- Dummy dump token is now UUID-shaped (`123e4567-...`) with a numeric
  `userId`, matching the real tokens the server accepts for JSON
  responses; a non-UUID token or non-numeric userId gets an empty
  HTTP 200 body from the server.
- One-liner launchers, `irm ... | iex` on Windows (`scripts/get.ps1`) and
  `curl ... | sh` on Linux (`scripts/get.sh`): download the latest release
  binary, verify its SHA-256 against the release's `SHA256SUMS`, run it,
  and clean up. On Linux the script re-runs under `sudo` only when
  `kernel.yama.ptrace_scope` blocks memory reads (`MARCONI_NO_SUDO=1`
  skips that and prints the manual hint). This is distribution only — the
  capture mechanism is unchanged, because Reverse:1999 does not cache the
  summon URL on disk (no Unity `webCaches`, empty Windows `INetCache`, and
  Lua `logNormal` lines are filtered from `logicLog` in the retail build).
- `.github/workflows/release.yml`: pushing a `v*` tag builds Linux and
  Windows release binaries, writes `SHA256SUMS`, and publishes them as a
  GitHub Release. The one-liners resolve
  `github.com/ArkInNight/marconi/releases/latest/download/...`, so they
  pick up each new release automatically.

### Fixed

- Tracker requests failing with `{"msg":"没有权限","code":-6}` ("no permission"):
  that response means the token in the captured link had expired (~290 s
  lifetime, see `SummonPoolHistoryModel:setToken`), not that capture
  failed. marconi now warns at capture time and in the README to import
  the link immediately and to re-grab a fresh link (exit Summon Review /
  restart the game, reopen, rerun) instead of reusing an old clipboard link.

### Changed

- URL validation now requires both `userId` and `token` (the only two
  parameters the game itself sends). Incomplete candidates found in memory
  are skipped instead of being handed to the tracker, where they could
  only fail server-side.
- `sanitize_url` normalizes the parameter order to the game's own
  `userId` first, then `token`, so trackers that prefix-match
  `.../query/summon?userId=` accept the link.

## [v0.4] - 2026-08-08

### Changed

- Memory scan chunk size reduced 8 MiB → 1 MiB and the poll interval raised
  2 s → 4 s, cutting peak RAM (~24 MiB buffer) and idle CPU while scanning.
- Clipboard: after copying, the process stays alive ~1 s on Linux. Wayland
  serves clipboard data on demand from the owning client, so exiting instantly
  could leave Klipper/Plasma with an empty clipboard.

### Added

- `make-dummy-dump` helper binary: writes a fake 4 MiB memory dump containing
  the summon URL as UTF-8 (straddling a chunk boundary) and UTF-16LE, for
  testing the full pipeline without launching the game:
  `marconi --dump-file /tmp/dummy.dmp`.

## [v0.3] - 2026-08-08

### Changed

- Memory scan now runs on a blocking thread, so Ctrl+C / SIGTERM and the
  scan timeout are handled promptly even while multi-GB regions are being read.
- Chunked scanning reuses buffers instead of allocating ~16 MB per chunk, and
  the anchor needles are cached once (`LazyLock`) instead of per call.
- Anchor search now uses the Two-Way `memmem` algorithm (`memchr` crate)
  instead of a naive sliding window — significantly faster on large dumps.
- Linux: adjacent readable regions with the same priority are merged before
  scanning, so strings spanning region boundaries are no longer missed.
- Linux: zombie processes are treated as dead instead of hanging until timeout.
- Linux/Windows: access-denied detection is now propagated even when it only
  happens mid-scan (not just on the first region), so the fix hint is shown.
- URL extraction returns `None` when no (properly aligned) scheme is found
  near the anchor, instead of emitting a malformed URL; UTF-16 scheme matches
  must share the anchor's 2-byte alignment.

### Added

- Windows memory scan now filters regions like the Linux backend: private
  (heap/stack) regions are scanned first, writable mapped files only after a
  few passes, and read-only images/mappings are skipped — matching the
  `include_file_mapped` policy.
- GitHub Actions CI: Linux job (fmt + clippy -D warnings + tests + release
  build) and a native Windows build job.

### Fixed

- Removed dead code (`AppError::NotFound/NotRunning/CaptureFailed`, unused
  `copy_failed` argument) and duplicated `CHUNK_SIZE`/`OVERLAP` constants.

## [v0.2] - 2026-08-05

### Changed

- Renamed project `r1999-link` → `marconi` (banner, CLI, runtime folders,
  binary name, test temp dirs, README, AGENT.md)
- Linux memory scan: regions that fail to read are skipped instead of aborting
  the whole scan
- Access-denied guidance message is now platform-aware (no ptrace/YAMA mention
  on Windows)

### Added

- Windows process detector (Toolhelp32 snapshot, case-insensitive match on
  `reverse1999` / `--game` pattern)
- Strategy 1: Windows memory scan (`OpenProcess` + `VirtualQueryEx` +
  `ReadProcessMemory`, readable `MEM_COMMIT` regions, chunked scan) — passes
  compile-check for `x86_64-pc-windows-gnu`, not yet runtime-tested on Windows
- CHANGELOG.md (this file)

### Notes

- End-to-end verification on Linux: a target process with the URL on the heap
  was scanned and the URL was captured.

## [v0.1] - 2026-08-05

### Added

- Rust skeleton: CLI, Output, Error, util (`exe_dir`, byte scanner)
- Cleanup Manager (runtime dirs + restore hooks + Drop, Ctrl+C/SIGTERM)
- Linux process detector (sysinfo + `/proc`, ancestor PID exclusion)
- Capture Engine trait + strategy orchestrator (memscan, proxy, hook)
- Strategy 1: Linux memory scan (ptrace + `process_vm_readv`, region priority,
  UTF-8 + UTF-16LE anchor scan, poll loop, `--dump-file` mode)
- URL Extractor (validates `/query/summon`, unescapes `&amp;`, sanitizes
  `userId` + `token`)
- Clipboard (arboard) with print fallback
- Unit tests (12)

### Linux notes

Reading another process's memory is restricted by `kernel.yama.ptrace_scope`
(default 1). When blocked, the program shows a one-time hint
(`sudo sysctl -w kernel.yama.ptrace_scope=0` or run with sudo). The change is
temporary (resets on reboot).

## Backlog (Next)

- Runtime-test the Windows backend on a real Windows machine (only cross
  compile-checked so far)
- Strategy 2: temporary HTTPS proxy + temporary CA
- Strategy 1b: native hook (WinHTTP/libcurl/OpenSSL)
- Auto launch game, GUI, auto upload, multi language
