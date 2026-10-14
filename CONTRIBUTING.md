# Contributing to Spoty 🎧

Thank you for your interest in contributing to **Spoty**! Spoty is an open-source, self-hosted Spotify desktop client clone built in Rust and Egui with a strict `<50MB RAM` budget and 100% data ownership.

We welcome contributions of all kinds: bug fixes, new storage providers, audio engine improvements, UI polish, documentation, and specifications.

---

## 1. Code of Conduct

We are committed to providing a welcoming, inclusive, and harassment-free environment for everyone. Please treat fellow contributors and maintainers with kindness, patience, and respect.

---

## 2. Prerequisites & Environment Setup

### 2.1 Rust Toolchain

Spoty requires **Rust 1.80+** (Rust 2024 edition). Install or update Rust via [rustup.rs](https://rustup.rs):

```bash
rustup update stable
```

### 2.2 System Audio & Build Dependencies

Spoty connects directly to native OS audio devices and rendering pipelines.

- **macOS:**
  Install the Xcode Command Line Tools:
  ```bash
  xcode-select --install
  ```

- **Linux (Ubuntu/Debian):**
  Install ALSA audio headers, udev libraries, and `pkg-config`:
  ```bash
  sudo apt-get update
  sudo apt-get install -y libasound2-dev libudev-dev pkg-config
  ```

- **Linux (Arch):**
  ```bash
  sudo pacman -S alsa-lib systemd pkgconf
  ```

- **Windows:**
  Install Visual Studio Build Tools with the "Desktop development with C++" workload.

---

## 3. Quickstart: Building & Running

1. **Clone the repository:**
   ```bash
   git clone https://github.com/your-username/spoty.git
   cd spoty
   ```

2. **Run in development mode:**
   ```bash
   cargo run -p spoty-ui
   ```

3. **Run with verbose debug logs:**
   ```bash
   RUST_LOG=spoty=debug cargo run -p spoty-ui
   ```

4. **Execute all workspace tests:**
   ```bash
   cargo test --workspace
   ```

---

## 4. Architecture & Crate Responsibilities

Spoty is organized into a modular Cargo workspace. When adding new features or fixing bugs, place your code in the appropriate crate and respect its boundaries:

```text
spoty/
├── crates/
│   ├── core/       # spoty-core: Domain models, IDs, playback state & queue engine
│   ├── storage/    # spoty-storage: Multi-backend storage, tag extraction, SQLite
│   ├── audio/      # spoty-audio: Background audio worker (rodio/cpal/symphonia)
│   └── ui/         # spoty-ui: Egui desktop interface, Spotify Encore theme & views
├── openspec/       # Living specifications & active change proposals
└── scripts/        # macOS packaging scripts
```

- **`crates/core` (`spoty-core`):**
  - **Responsibilities:** Core data structures (`Track`, `Album`, `Artist`, `Playlist`), queue management (`QueueManager`), and playback states.
  - **Boundary:** Pure domain logic only. Zero filesystem, network, or GUI dependencies.
- **`crates/storage` (`spoty-storage`):**
  - **Responsibilities:** Storage providers (`OpenDalProvider`, `SubsonicProvider`, `SpotifyProvider`), audio tag/art parser (`lofty`), and local SQLite database (`rusqlite`).
  - **Boundary:** Independent of the audio playback engine and UI.
- **`crates/audio` (`spoty-audio`):**
  - **Responsibilities:** Dedicated OS thread for Rodio/CPAL playback, Symphonia stream decoding, timeline tracking, and live audio visualizer FFT buffer.
  - **Boundary:** Communicates with the UI strictly via lockless/non-blocking command channels.
- **`crates/ui` (`spoty-ui`):**
  - **Responsibilities:** Egui views (`HomeView`, `PlaylistView`, `SearchView`, `SettingsView`), Spotify Encore theme styling, custom widgets, and keyboard shortcuts.
  - **Boundary:** Must **never** execute blocking operations (network requests, large scans) on the UI thread.

---

## 5. Spec-Driven Development with OpenSpec

For non-trivial features, architectural updates, or behavior modifications, Spoty practices **Spec-Driven Development** using **OpenSpec**.

### 5.1 When is an OpenSpec Proposal Needed?
- Adding a new capability or storage provider.
- Modifying playback engine behavior or queue mechanics.
- Significant UI redesigns or new major views.
- Minor bug fixes, typo corrections, or small documentation adjustments do **not** require a full OpenSpec proposal.

### 5.2 The OpenSpec Workflow
1. **Create a Change Proposal:**
   Create a folder under `openspec/changes/<change-name>/` containing:
   - `proposal.md`: Explains the motivation, what changes, affected capabilities, and architectural impact.
   - `design.md`: Technical design, contracts, interfaces, and trade-offs.
   - `tasks.md`: Checkable list of implementation steps (`- [ ]`).
   - `specs/<capability>/spec.md`: Delta specifications using formal requirement keywords (`SHALL`, `MUST`) and `GIVEN`/`WHEN`/`THEN` scenarios.
2. **Implement & Check Off Tasks:**
   Implement code incrementally, adding tests alongside your implementation and updating `tasks.md` with `[x]`.
3. **Sync & Archive:**
   Once verified, delta specifications are synchronized into `openspec/specs/` and the change is archived under `openspec/changes/archive/YYYY-MM-DD-<change-name>/`.

---

## 6. Coding Standards & Conventions

### 6.1 Formatting & Linting
All code submitted to Spoty must pass formatting and linter checks:

- **Format code:**
  ```bash
  cargo fmt --all
  ```
- **Check formatting without modifying:**
  ```bash
  cargo fmt --all -- --check
  ```
- **Run Clippy:**
  ```bash
  cargo clippy --workspace --all-targets
  ```

### 6.2 Error Handling
- In library crates (`spoty-core`, `spoty-storage`, `spoty-audio`), use [`thiserror`](https://docs.rs/thiserror) to define explicit, typed error enums.
- In application crates (`spoty-ui`), use [`anyhow`](https://docs.rs/anyhow) for flexible context propagation where appropriate.
- Avoid `.unwrap()` and `.expect()` in non-test code paths. Handle potential failures gracefully or log them using `tracing::warn!` or `tracing::error!`.

### 6.3 UI & Theming Standards
- UI code in `crates/ui` must adhere to the Spotify Encore design tokens defined in `crates/ui/src/theme.rs`.
- Do not hardcode arbitrary hex colors; use the established constants (`COLOR_CANVAS_BLACK`, `COLOR_CARD_DARK`, `COLOR_SPOTIFY_GREEN`, `COLOR_TEXT_PRIMARY`, etc.).
- Maintain responsive, keyboard-accessible layouts.

### 6.4 Concurrency & UI Responsiveness
- The Egui rendering loop runs at 60 FPS. Never make blocking network calls or synchronous heavy disk reads directly inside `eframe::App::update`.
- Offload expensive tasks to background threads or Tokio tasks, and communicate results via channels (`std::sync::mpsc` or `crossbeam_channel`).

---

## 7. Git & Pull Request Guidelines

### 7.1 Branch Naming
Create focused, descriptive branches from `main`:
- `feat/spotify-connect-provider`
- `fix/queue-shuffle-repeat-boundary`
- `docs/clarify-s3-endpoints`
- `spec/audio-equalizer`

### 7.2 Conventional Commits
Spoty follows the [Conventional Commits](https://www.conventionalcommits.org/) specification:

- `feat:` A new feature or user-facing capability.
- `fix:` A bug fix.
- `docs:` Documentation changes only.
- `docs(openspec):` Changes to specifications, proposals, or archived changes.
- `refactor:` Code changes that neither fix a bug nor add a feature.
- `test:` Adding or updating tests.
- `chore:` Maintenance, dependency updates, or packaging scripts.

**Example:**
```bash
git commit -m "feat(storage): support custom S3 endpoints for MinIO"
```

### 7.3 Pre-Flight Checklist Before Submitting
Before pushing your branch and opening a pull request, run the following:

```bash
# 1. Format code
cargo fmt --all

# 2. Check compiler warnings & lints
cargo clippy --workspace

# 3. Run all tests
cargo test --workspace
```

### 7.4 Opening a Pull Request
1. Push your branch to your fork on GitHub.
2. Open a Pull Request against Spoty's `main` branch.
3. Complete the pull request template (`.github/pull_request_template.md`), including:
   - Clear summary of changes.
   - Reference to the associated OpenSpec change or issue.
   - Checked crates affected.
   - Test plan and manual verification steps.
   - Screenshots or audio behavior notes if UI or playback was modified.

Thank you for helping make Spoty the premier open-source Spotify alternative! 💚
