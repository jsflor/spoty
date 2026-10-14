## 1. Queue Management (`spoty-audio` or `spoty-core`)

- [x] 1.1 Implement `Queue` in `spoty-core` with track list, current index, shuffle index permutation, and `next()` / `prev()` methods.
- [x] 1.2 Add unit tests for `Queue` verifying sequential progression, repeat all wrapping, and shuffle ordering. Verify with `cargo test -p spoty-core`.

## 2. Navigation & Views (`spoty-ui`)

- [x] 2.1 Implement `Route` and `NavigationHistory` with `push()`, `back()`, and `forward()`.
- [x] 2.2 Implement `HomeView`: Greeting banner, recent playlist cards with hover play button, and album grid.
- [x] 2.3 Implement `SearchView`: Search bar, instant track/artist matching list, and colorful category browse tiles.
- [x] 2.4 Implement `PlaylistView`: Hero banner with dominant background gradient, total duration summary, and track list table.
- [x] 2.5 Implement `SettingsView`: Storage provider switcher (Demo catalog vs Local Directory via `OpenDalProvider`).

## 3. Integration & Workspace Verification

- [x] 3.1 Connect top bar navigation arrows (`◀` / `▶`) to history stack state and buttons.
- [x] 3.2 Connect player bar Next (`⏭`) and Previous (`⏮`) buttons and auto-play on `TrackEnded` event to the `Queue`.
- [x] 3.3 Run full workspace build and test suite. Verify with `cargo test --workspace`.
