# Marconi — Reverse:1999 Link Grabber

CLI lintas platform untuk mengambil URL **Summon History** (`/query/summon`) dari Reverse:1999 secara otomatis — tanpa Fiddler, tanpa mengubah sistem secara permanen.

Nama diambil dari **Guglielmo Marconi**, penemu radio: seperti Marconi menangkap sinyal dari udara, tool ini menangkap URL summon dari game.

## Cara pakai

```sh
./marconi
```

1. Jalankan program.
2. Buka Reverse:1999 (Windows/Steam Proton).
3. Buka halaman **Summon Review** / Summon History di dalam game.
4. URL otomatis terdeteksi, disalin ke clipboard, lalu program keluar.

Contoh output:

```
==================================

Reverse1999 Link Grabber

==================================

Waiting for Reverse:1999...

Game detected. (pid: 30680, Reverse1999.exe)

Waiting for summon history...

Open the Summon Review page in-game when ready.

✔ Link Found

https://game-re-en-service.sl916.com/query/summon?userId=...&token=...

Copied to clipboard.

Done.
```

## Opsi CLI

| Opsi | Keterangan |
| --- | --- |
| `--game <pola>` | Timpa pola pencarian proses game (substring, case-insensitive). |
| `--timeout <detik>` | Waktu tunggu URL setelah game terdeteksi (default 300). |
| `--game-timeout <detik>` | Waktu tunggu game menyala (0 = tunggu terus). |
| `--dump-file <path>` | Scan file dump memori alih-alih proses hidup (debug/testing). |
| `--no-clipboard` | Jangan salin ke clipboard. |
| `-v, --verbose` | Output rinci. |

## Cara kerja

1. **Process Detector** menemukan proses game (`Reverse1999.exe` / Steam `3092660`).
2. **Capture Engine (Strategi 1: memory scan)** menempel ke proses (`ptrace` + `process_vm_readv`), membaca region memori, mencari string `com/query/summon` (UTF-8 **dan** UTF-16LE), lalu mengambil URL lengkap di sekitarnya.
3. **URL Extractor** memvalidasi URL (`/query/summon` + parameter), membersihkan `&amp;`, dan menyisakan parameter `userId` + `token` (format yang dipakai gacha tracker).
4. **Clipboard** menyalin URL; jika gagal, URL tetap ditampilkan.
5. **Cleanup Manager** menghapus direktori runtime (`cache/ temp/ logs/`) dan memulihkan semua perubahan pada saat keluar (termasuk Ctrl+C / SIGTERM).

Semua file sementara dibuat di dalam folder program dan dibersihkan saat selesai. Tidak ada data yang dikirim ke mana pun.

## Catatan Linux / Proton

- Membaca memori proses lain membutuhkan akses `ptrace`. Distribusi dengan
  `kernel.yama.ptrace_scope=1` (default Arch/Fedora/Ubuntu) memblokir attach
  ke proses yang bukan anak dari program ini. Solusi sementara:

  ```sh
  sudo sysctl -w kernel.yama.ptrace_scope=0
  ```

  (berlaku sampai reboot; atau jalankan tool dengan `sudo`). Jika tetap
  terblokir, program menampilkan pesan panduan otomatis.

## Status platform

- **Linux (Proton)** — aktif (Strategi 1 memory scan).
- **Windows** — kerangka modul sudah ada; implementasi proses-deteksi & memory-read menyusul (Phase 2).

## Pengembangan

```sh
cargo build --release
cargo test
cargo clippy --all-targets
```

Arsitektur mengikuti `AGENT.md`: `CLI`, `Process Detector`, `Capture Engine`
(trait + orkestrator strategi), `URL Extractor`, `Clipboard`, `Cleanup Manager`,
`Output`. Strategi berikutnya (temporary HTTPS proxy, native hook) tersedia
sebagai stub di belakang trait yang sama.
