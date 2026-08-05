# Changelog

All notable changes to the Marconi project are recorded in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

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
