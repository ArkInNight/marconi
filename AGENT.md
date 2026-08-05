# Reverse: 1999 Link Grabber

Version: 0.2

---

# 1. Project Vision

## Goal

Membangun aplikasi CLI lintas platform yang dapat mengambil URL **Summon History** (`/query/summon`) dari Reverse: 1999 secara otomatis tanpa memerlukan Fiddler.

Target utama adalah menghasilkan pengalaman pengguna sesederhana mungkin:

```
Windows

marconi.exe
```

atau

```
Linux

./marconi
```

Kemudian pengguna hanya membuka **Summon History**, dan aplikasi langsung menampilkan URL yang dapat ditempel ke web Gacha Tracker.

---

# 2. Project Goals

## Functional Goals

- Menemukan URL `/query/summon`
- Menampilkan URL ke terminal
- Menyalin URL ke clipboard
- Keluar secara otomatis setelah URL ditemukan

## User Experience Goals

Tidak ada konfigurasi manual.

Tidak ada langkah seperti:

- membuka Fiddler
- mencari request
- copy URL dari network log

User hanya menjalankan program.

---

# 3. Non Functional Requirements

## Zero Installation

Idealnya cukup:

- download
- extract
- run

Tidak memerlukan installer.

---

## Zero Permanent Changes

Program tidak boleh meninggalkan perubahan permanen pada sistem.

Tidak boleh:

- mengubah registry permanen
- mengubah proxy permanen
- mengubah environment variable permanen
- membuat service
- membuat scheduled task
- membuat startup process
- menginstall dependency sistem

Semua perubahan harus bersifat sementara dan dipulihkan sebelum program keluar.

---

## Self Contained

Semua file yang dibuat program harus berada di dalam folder project.

Contoh:

```
Reverse1999-LinkGrabber/

    bin/
    cache/
    temp/
    cert/
    logs/

    marconi.exe
    marconi
```

Tidak boleh membuat folder baru di luar direktori project kecuali benar-benar diperlukan oleh sistem operasi.

---

# 4. Supported Platforms

## Phase 1

Windows 10

Windows 11

---

## Phase 2

Linux

Target utama:

- Arch Linux
- Fedora
- Ubuntu

Game dijalankan menggunakan:

- Steam Proton
- Proton GE

---

# 5. User Flow

```
Run program

↓

Waiting for game

↓

Game detected

↓

Waiting summon history

↓

User opens summon history

↓

URL detected

↓

Copy to clipboard

↓

Print URL

↓

Exit
```

---

# 6. Output

Contoh:

```
==================================

Reverse1999 Link Grabber

==================================

Waiting for Reverse:1999...

Game detected.

Waiting for summon history...

✔ Link Found

https://xxxxxxxx/query/summon?...

Copied to clipboard.

Done.
```

---

# 7. Cleanup Policy

Saat program selesai:

- hapus temporary file
- hapus log sementara
- hentikan proses internal
- restore seluruh konfigurasi yang sempat diubah

Program harus aman walaupun user menutup dengan:

- Ctrl+C
- SIGTERM
- crash ringan

Cleanup tetap dijalankan.

---

# 8. Architecture

```
CLI

│

├── Process Detector
│
├── Capture Engine
│
├── URL Extractor
│
├── Clipboard
│
├── Cleanup Manager
│
└── Output
```

---

# 9. Modules

## CLI

Tanggung jawab:

- parsing argument
- menampilkan progress
- exit code

---

## Process Detector

Tugas:

- mendeteksi Reverse1999 berjalan
- mendapatkan PID
- memonitor proses

---

## Capture Engine

Komponen inti.

Tanggung jawab:

- menangkap request HTTPS
- menemukan request Summon History
- mengirim URL ke URL Extractor

Capture Engine dibuat menggunakan interface sehingga implementasi dapat diganti tanpa mengubah modul lain.

```
CaptureEngine

↓

Found URL

↓

URL Extractor
```

---

## URL Extractor

Memastikan URL valid.

Harus:

- mengandung `/query/summon`
- mempunyai parameter yang diperlukan
- bukan request lain

---

## Clipboard

Menyalin URL secara otomatis.

Jika clipboard gagal, URL tetap ditampilkan di terminal.

---

## Cleanup Manager

Menghapus:

- file sementara
- log sementara
- cache sementara

Mengembalikan seluruh konfigurasi sistem yang sempat berubah.

---

# 10. Capture Strategy

Capture Engine menggunakan strategi bertingkat.

## Strategy Priority

### Strategy 1

Native Hook

Target:

- WinHTTP
- WinINet
- libcurl
- OpenSSL
- TLS library lain yang digunakan game

Keuntungan:

- tidak memerlukan proxy
- tidak memerlukan certificate
- tidak mengubah sistem

Target utama project.

---

### Strategy 2

Temporary HTTPS Proxy

Digunakan hanya jika Strategy 1 gagal.

Syarat:

- seluruh konfigurasi proxy dipulihkan saat selesai
- seluruh certificate bersifat sementara
- tidak meninggalkan perubahan permanen

---

### Strategy 3

Fallback Manual

Jika seluruh metode otomatis gagal.

Program hanya memberikan instruksi minimal kepada user.

Strategi ini dihindari sebisa mungkin.

---

# 11. Development Principles

Seluruh modul harus:

- loosely coupled
- mudah diuji
- mudah diganti implementasinya
- memiliki interface yang jelas

Capture Engine tidak boleh bergantung pada implementasi CLI.

---

# 12. Technology Stack

Bahasa utama:

Rust

Library yang direkomendasikan:

- clap
- arboard
- anyhow
- tokio
- serde
- serde_json

Library tambahan dipilih seminimal mungkin.

---

# 13. Security

Program tidak boleh:

- mengirim data ke server pihak ketiga
- menyimpan URL summon
- menyimpan token
- menyimpan history

Seluruh data hanya diproses secara lokal.

---

# 14. Future Features

- Auto launch game
- GUI mode
- Auto upload ke web tracker
- Auto update
- Multi language
- Plugin capture engine
- Support game lain
- Export diagnostic log
- Portable build

---

# 15. Success Criteria

Project dianggap berhasil apabila:

- User hanya menjalankan satu file.
- URL `/query/summon` berhasil didapat tanpa Fiddler.
- URL otomatis tersalin ke clipboard.
- Program keluar otomatis.
- Tidak meninggalkan perubahan permanen pada sistem.
- Semua file sementara berada di dalam folder project dan dibersihkan saat program selesai.

---

# 16. Changelog

Perubahan per versi, status implementasi, dan backlog tercatat di
[CHANGELOG.md](./CHANGELOG.md).

