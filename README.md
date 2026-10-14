# Spoty 🎧

An open-source, self-hosted Spotify desktop client clone built with **Rust** and **Egui**.

Spoty replicates Spotify's desktop user experience (Encore dark theme, 3-panel layout, playlist hero banners, queue management, and live audio visualizer) while consuming **<50MB of RAM** (compared to 500MB–1GB in Electron/CEF). Most importantly, it gives you **100% data ownership**: stream your personal music collection from local hard drives, cloud buckets, or home servers.

---

## Features

- **Spotify Encore Visuals:** Dark theme hierarchy (`#121212`, `#181818`, `#242424`, `#1ed760`), responsive layout, and custom slider widgets.
- **Real-Time Audio Spectrum Visualizer:** 16-band live equalizer animating in Spotify green in the playback bar.
- **Embedded Tag & Art Extraction:** Pure-Rust ID3v2, Vorbis, and FLAC tag parsing (`lofty`) with base64 embedded cover art extraction.
- **Zero-Latency Database Caching:** Embedded SQLite database (`library.db`) with instant startup, persistent favorites (`💚`), and custom playlist creation (`＋`).
- **Desktop Ergonomics:** Full keyboard shortcuts (`Space`, `Arrows`, `M`, `S`, `R`).
- **Flexible Music Sources:**
  1. 🎵 **Demo Catalog:** Zero-config synthesized 44.1kHz stereo PCM WAV tracks for instant testing.
  2. 📁 **Local Storage (OpenDAL):** Scan folders on your Mac, PC, external SSD, or home NAS.
  3. ☁️ **Cloud Storage (OpenDAL S3/R2):** Direct streaming from Cloudflare R2, AWS S3, MinIO, or Backblaze B2.
  4. 🥧 **Home Music Servers:** Native Navidrome / Subsonic / Jellyfin protocol support.
  5. 🟢 **Official Spotify:** Sync playlists and search millions of tracks via Spotify Web API.

---

## 1. Local Development Setup

### Prerequisites

- **Rust & Cargo:** Rust 1.80+ (2024 edition). Install via [rustup.rs](https://rustup.rs):
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```
- **macOS:** Xcode Command Line Tools:
  ```bash
  xcode-select --install
  ```
- **Linux (Ubuntu/Debian):** ALSA audio and udev development headers:
  ```bash
  sudo apt-get update && sudo apt-get install -y libasound2-dev libudev-dev pkg-config
  ```

### Building & Running from Source

1. **Enter directory:**
   ```bash
   cd spoty
   ```

2. **Run in Development Mode:**
   ```bash
   cargo run -p spoty-ui
   ```

3. **Run with Debug Logging:**
   ```bash
   RUST_LOG=spoty=debug cargo run -p spoty-ui
   ```

4. **Run the Workspace Test Suite:**
   ```bash
   cargo test --workspace
   ```

### Packaging for macOS (`Spoty.app`)

To generate a standalone macOS Application Bundle with release optimizations:
```bash
./scripts/package-macos.sh

# Install to Applications
cp -r target/dist/Spoty.app /Applications/
open /Applications/Spoty.app
```

---

## 2. Self-Hosting & Local Usage Guide

Spoty allows you to choose where your music files are hosted without altering the player UI.

### Option A: Local Storage (Mac, PC, or Mounted Network Share)
1. Open **Settings** (`⚙`) in Spoty's sidebar.
2. In **1. Local Storage (OpenDAL)**, enter the path to your music directory (e.g. `/Users/yourname/Music` or `/media/nas/music`).
3. Click **"Scan & Mount Folder"**.
4. Spoty will recursively index your audio files (`.mp3`, `.flac`, `.wav`, `.ogg`, `.m4a`), parse embedded ID3/FLAC tags and cover art, and save them to your local SQLite database for instant access.

### Option B: Cloudflare R2 / AWS S3 Object Storage
Store your FLAC/MP3 files in high-durability cloud storage (e.g. Cloudflare R2 has zero egress fees):
1. Create a bucket in your Cloudflare or AWS dashboard and upload your music folders.
2. In Spoty, navigate to **Settings** (`⚙`) ➔ **2. S3 & Cloudflare R2**.
3. Fill in your bucket details:
   - **Bucket Name:** `my-personal-music`
   - **Endpoint:** `https://<account_id>.r2.cloudflarestorage.com` (for Cloudflare R2) or standard AWS S3 endpoint
   - **Access Key ID & Secret Access Key:** Your read-only S3 API token.
4. Click **"Connect & Mount S3 Bucket"**. Spoty streams tracks on-demand without downloading the entire collection to your local machine.

### Option C: Home Server with Navidrome / Subsonic (Raspberry Pi / NAS)
If you run a self-hosted media server on a Raspberry Pi or home server using Docker:
```bash
docker run -d \
   --name navidrome \
   -v /path/to/music:/music:ro \
   -v /path/to/data:/data \
   -p 4533:4533 \
   -e ND_SCANSCHEDULE=1h \
   deluan/navidrome:latest
```
In Spoty:
1. Navigate to **Settings** (`⚙`) ➔ **3. Navidrome / Subsonic Home Server**.
2. Enter:
   - **Server URL:** `http://raspberrypi.local:4533` (or your domain/tailscale IP)
   - **Username & Password**
3. Click **"Test Connection & Sync Library"**. Spoty authenticates using MD5 token/salt encryption and imports your server's playlists and library.

### Option D: Official Spotify Account
1. Create a free app at [developer.spotify.com/dashboard](https://developer.spotify.com/dashboard) to get a Client ID and Secret.
2. In Spoty **Settings** (`⚙`), enter your Client ID and Client Secret under **4. Official Spotify Integration**.
3. Click **"Connect & Sync Spotify"** to search Spotify's catalog and stream audio previews directly within Spoty.

---

## Configuration & Local Persistence

Spoty stores state automatically in standard system directories:
- **Configuration:** `~/.config/spoty/config.json` (active provider, endpoints, credentials).
- **SQLite Database:** `~/.config/spoty/library.db` (cached tracks, user playlists, favorites).

---

## Desktop Keyboard Shortcuts

| Shortcut | Action |
|---|---|
| `Space` | Toggle Play / Pause |
| `Right Arrow` | Seek Forward 5 seconds |
| `Shift + Right Arrow` | Skip to Next track |
| `Left Arrow` | Seek Backward 5 seconds |
| `Shift + Left Arrow` | Skip to Previous track |
| `Up Arrow` | Volume Up (+5%) |
| `Down Arrow` | Volume Down (-5%) |
| `M` | Toggle Mute |
| `S` | Toggle Shuffle |
| `R` | Cycle Repeat Mode (`Off` ➔ `All` ➔ `One`) |

*(Keyboard shortcuts are automatically suppressed while typing in Search or Settings text fields).*

---

## Architecture Overview

```text
spoty/
├── crates/
│   ├── core/       # Domain models, IDs, playback state & queue engine (spoty-core)
│   ├── storage/    # Multi-backend storage (OpenDAL FS/S3, Subsonic, Spotify, SQLite) (spoty-storage)
│   ├── audio/      # Background audio worker on OS thread (rodio/cpal sink) (spoty-audio)
│   └── ui/         # Egui desktop UI, Spotify Encore theme, router & views (spoty-ui)
├── openspec/       # 14 living specifications & change archives
└── scripts/        # macOS packaging scripts
```

---

## License

Dual-licensed under either of:
- Apache License, Version 2.0 ([LICENSE-APACHE](http://www.apache.org/licenses/LICENSE-2.0))
- MIT license ([LICENSE-MIT](http://opensource.org/licenses/MIT))
