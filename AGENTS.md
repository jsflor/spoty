# AGENTS.md — Spoty AI Agent Operations & Spec-Driven Development Guide

Welcome to **Spoty**. This document defines the engineering standards, technical architecture, tool catalog, and **Spec-Driven Development (OpenSpec)** workflow that all AI agents operating in this repository MUST adhere to.

---

## 1. Project Mission & Identity

Spoty is an open-source, self-hosted Spotify desktop client clone built in **Rust** with **Egui**.

- **RAM Footprint:** Strictly `<50MB RAM` under normal operating conditions (compared to 500MB–1GB+ in Electron/CEF).
- **100% Data Ownership:** Playback unification across personal storage (Local files via OpenDAL, AWS S3 / Cloudflare R2), home servers (Navidrome / Subsonic / Jellyfin), and official Spotify Web API previews.
- **Design Authenticity:** High-fidelity recreation of Spotify's desktop user experience (Spotify Encore dark theme, "Cards on Canvas" 3-panel layout, playlist hero banners, custom sliders, and real-time audio visualizer).

---

## 2. Multi-Crate Architecture & Isolation Boundaries

Spoty is structured as a Cargo workspace with four specialized crates. Strict architectural isolation between crates MUST be preserved.

```text
spoty/
├── crates/
│   ├── core/       # spoty-core: Domain models, queue logic, playback state machines
│   ├── storage/    # spoty-storage: Multi-backend storage, tag extraction, SQLite cache
│   ├── audio/      # spoty-audio: Background OS audio worker, stream decoding, sink
│   └── ui/         # spoty-ui: Egui desktop client, views router, Encore theme
├── openspec/       # Living specs, active change proposals, and historical archives
└── scripts/        # Standalone macOS packaging & release helpers
```

### Architectural Commandments

| Crate | Primary Responsibilities | Strict Constraints / Prohibitions |
|---|---|---|
| **`crates/core`** (`spoty-core`) | Core models (`Track`, `Album`, `Artist`, `Playlist`), strongly-typed IDs (`TrackId`, `PlaylistId`), queue management (`QueueManager`, repeat/shuffle), playback state. | **ZERO I/O ALLOWED.** No disk access, no network requests, no audio output, no GUI imports. Pure, deterministic domain logic only. |
| **`crates/storage`** (`spoty-storage`) | `StorageProvider` trait and implementations: `OpenDalProvider` (local FS & S3/R2), `SubsonicProvider`, `SpotifyProvider`, `DemoProvider`. Metadata parsing (`lofty`), SQLite database (`rusqlite`). | Decoupled from GUI and playback sinks. Must not depend on `spoty-audio` or `spoty-ui`. Return data via standard `core` models. |
| **`crates/audio`** (`spoty-audio`) | Background OS audio thread worker (`rodio`, `cpal`, `symphonia`), non-blocking command channel (`PlayerController`), event dispatching (`PlayerEvent`), live FFT/sample ringbuffer for visualizer. | Must run decoding and audio output on a dedicated OS thread. Must never block or execute on the UI rendering thread. Do not import `spoty-ui`. |
| **`crates/ui`** (`spoty-ui`) | Immediate-mode desktop interface using `eframe` (wgpu) & `egui`. Spotify Encore dark theme (`theme.rs`), custom widgets (`widgets/slider.rs`, `widgets/visualizer.rs`), views router (`views/*`), keyboard shortcuts. | **NEVER perform blocking I/O on the rendering loop.** All network scans, database writes, and heavy tag extraction must be dispatched asynchronously or across background channels. |

---

## 3. Technology & Dependency Catalog

All AI agents must keep dependency usage consistent with the existing workspace configuration:

- **Rust Toolchain:** Rust 2024 edition, Rust 1.80+.
- **GUI Framework:**
  - `eframe` (v0.31) with `wgpu` rendering backend and `default_fonts`.
  - `egui` (v0.31) immediate-mode GUI.
  - `egui_extras` (v0.31) with image and font loaders.
  - `image` (v0.25) for PNG/JPEG image decoding.
- **Audio Decoding & Synthesis:**
  - `rodio` (v0.20) for device sinks, playback control, and sample conversion.
  - `symphonia` (v0.5) for broad container and codec format support (MP3, FLAC, AAC, Vorbis, WAV).
- **Storage & Ingestion:**
  - `opendal` (v0.59) with `services-fs` and `services-s3` features for unified object and file storage.
  - `lofty` (v0.25) for pure-Rust ID3v1/v2, Vorbis, and FLAC tag/picture extraction.
  - `rusqlite` (v0.40) with `bundled` SQLite 3 for instant-startup local metadata, favorites (`💚`), and custom playlists.
  - `ureq` for lightweight synchronous HTTP requests with Subsonic MD5 salt authentication and Spotify Web API OAuth token exchange.
- **Serialization & Concurrency:**
  - `serde`, `serde_json` for serialization/deserialization.
  - `thiserror` (v2.0) for crate-level strongly-typed error definitions; `anyhow` for top-level application error context.
  - `tokio` (v1) async runtime where applicable.
  - `tracing` & `tracing-subscriber` for structured logging (`RUST_LOG=spoty=debug`).

---

## 4. Spec-Driven Development Workflow (OpenSpec)

Spoty uses **OpenSpec** for all non-trivial features, refactorings, and architectural additions. The specification is the single source of truth for requirements, behavior, and progress.

### 4.1 OpenSpec Repository Layout

```text
openspec/
├── config.yaml               # OpenSpec schema & operational guidelines
├── specs/                    # Living specifications representing current system truth
│   ├── core/domain-model/spec.md
│   ├── audio/playback-engine/spec.md
│   ├── audio/queue-management/spec.md
│   ├── storage/provider-abstraction/spec.md
│   ├── storage/opendal-provider/spec.md
│   ├── storage/s3-cloud-storage/spec.md
│   ├── storage/subsonic-provider/spec.md
│   ├── storage/spotify-provider/spec.md
│   ├── storage/sqlite-cache-and-favorites/spec.md
│   ├── storage/metadata-tag-extraction/spec.md
│   ├── ui/navigation-and-views/spec.md
│   ├── ui/spotify-player-bar/spec.md
│   ├── ui/audio-visualizer/spec.md
│   ├── ui/keyboard-shortcuts/spec.md
│   └── dist/macos-packaging/spec.md
└── changes/                  # Active and archived change proposals
    ├── <active-change-name>/ # In-flight feature work
    │   ├── proposal.md       # Problem statement, capabilities, and impact
    │   ├── design.md         # Technical design, crate contracts, decisions
    │   ├── tasks.md          # Ordered, checkable implementation plan
    │   └── specs/            # Delta specs (new or modified capabilities)
    └── archive/              # Permanently archived completed changes
        └── YYYY-MM-DD-<change-name>/
```

### 4.2 Lifecycle of a Change

When an agent is tasked with adding a capability or modifying behavior, the following workflow MUST be applied:

```
[Propose] ➔ [Author Specs & Tasks] ➔ [Apply Tasks & Code] ➔ [Verify Gates] ➔ [Sync & Archive]
```

#### Step 1: Propose
- Create `openspec/changes/<change-name>/proposal.md`.
- Detail the **Why** (problem/motivation), **What Changes**, **Capabilities** (new/modified), and **Impact**.

#### Step 2: Design & Tasks
- Create `openspec/changes/<change-name>/design.md` detailing crate interactions, structs, traits, and trade-offs.
- Create `openspec/changes/<change-name>/tasks.md` with numbered, bite-sized tasks formatted as checkable Markdown items (`- [ ] task`).
- Author delta specifications under `openspec/changes/<change-name>/specs/<capability>/spec.md` using formal RFC 2119 keywords (`SHALL`, `MUST`, `SHOULD`) and concrete `GIVEN` / `WHEN` / `THEN` scenarios.

#### Step 3: Apply
- Implement tasks systematically in order.
- Follow test-driven development: write unit or integration tests verifying the behavior alongside the implementation.
- Mark each completed item in `tasks.md` as checked (`- [x]`).

#### Step 4: Verify Quality Gates
- Execute the workspace test suite:
  ```bash
  cargo test --workspace
  ```
- Run formatting and compiler checks:
  ```bash
  cargo fmt --all -- --check
  cargo check --workspace
  ```

#### Step 5: Sync Specs & Archive
- Merge delta specs into the living specifications under `openspec/specs/`.
- Move the completed change from `openspec/changes/<change-name>` to `openspec/changes/archive/YYYY-MM-DD-<change-name>`.

---

## 5. Rules & Commandments for AI Agents

1. **Do Not Bypass OpenSpec:** Any new feature or cross-crate change must have a corresponding change proposal in `openspec/changes/` before editing code.
2. **Never Break Compilation or Tests:** Before considering a task complete, verify with `cargo test --workspace`. All existing tests must pass.
3. **Preserve Crate Boundaries:** Never import `spoty-ui` into `spoty-core`, `spoty-storage`, or `spoty-audio`. Keep `spoty-core` completely zero-I/O.
4. **Never Block the Egui Main Loop:** Any operation that can take `>16ms` (disk indexing, network requests, audio decoding) must run on a background thread or async channel.
5. **Honor Spotify Encore Theme Tokens:** Do not introduce arbitrary colors or hardcoded hex values in UI widgets. Use the centralized constants and semantic styles in `crates/ui/src/theme.rs`.
6. **Follow Conventional Commits:** When creating git commits, use the conventional commit specification (`feat:`, `fix:`, `refactor:`, `test:`, `docs:`, `docs(openspec):`, `chore:`).
