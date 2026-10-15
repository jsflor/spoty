# Tasks: home-horizontal-carousels

## 1. Carousel Scroll Engine (`crates/ui/src/views/home.rs`)

- [x] 1.1 Replace `ui.horizontal_wrapped` with `egui::ScrollArea::horizontal()` in `HomeView`.
- [x] 1.2 Implement interactive chevron navigation buttons (`‹` / `›`) in carousel section headers.
- [x] 1.3 Add persistent carousel scroll offset state to support smooth paging.

## 2. Multi-Section Carousels (`crates/ui/src/views/home.rs`)

- [x] 2.1 Refactor "Featured & Made For You" track cards into a single-row horizontal carousel.
- [x] 2.2 Add "Your Playlists & Mixes" horizontal carousel displaying full-size playlist cards.
- [x] 2.3 Add "Artists" horizontal carousel displaying circular artist avatar cards.
- [x] 2.4 Verify compilation with `cargo check -p spoty-ui`.

## 3. Verification and Quality Gates

- [x] 3.1 Run full workspace test suite `cargo test --workspace`.
- [x] 3.2 Verify formatting and compiler checks with `cargo fmt --all -- --check` and `cargo check --workspace`.
