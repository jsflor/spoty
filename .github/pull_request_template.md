## Description
<!-- Provide a clear, concise summary of the changes introduced by this pull request and the motivation behind them. -->

## Associated OpenSpec Change / Issue
- OpenSpec Change: `openspec/changes/<change-name>` (or N/A if minor fix)
- Closes / Fixes: #<!-- Issue number, if applicable -->

## Type of Change
- [ ] 🚀 New feature (non-breaking change introducing a new capability)
- [ ] 🐛 Bug fix (non-breaking change addressing an issue)
- [ ] 🎨 UI / Visual polish (Spotify Encore styling, widget refinements)
- [ ] ⚡ Performance optimization (memory footprint, audio latency, DB indexing)
- [ ] ♻️ Refactoring / Architectural cleanup
- [ ] 📝 Documentation / OpenSpec specification update

## Crates Affected
- [ ] `crates/core` (`spoty-core` - Domain models, queue logic, playback state)
- [ ] `crates/storage` (`spoty-storage` - OpenDAL, S3/R2, Subsonic, Spotify, SQLite, Metadata)
- [ ] `crates/audio` (`spoty-audio` - Rodio audio worker, stream decoding, playback controller)
- [ ] `crates/ui` (`spoty-ui` - egui interface, views, widgets, theme tokens)
- [ ] `openspec/` (Specifications, proposals, task tracking)
- [ ] `scripts/` (Packaging, CI/CD, build tooling)

## Verification & Testing

### Automated Tests
```bash
cargo test --workspace
```
- [ ] All workspace tests passed cleanly.

### Manual Verification
<!-- Outline the manual steps performed to verify this change (e.g., playback testing, UI responsiveness, provider mounting). -->
1. 
2. 
3. 

## UI / Audio Preview (if applicable)
<!-- Attach screenshots, screen recordings, or describe audio behavior if this PR touches the GUI or audio pipeline. -->

## Contributor Checklist
- [ ] My code adheres to the project's coding standards in `CONTRIBUTING.md`.
- [ ] Code is formatted with `cargo fmt --all`.
- [ ] Clippy checks pass without new warnings (`cargo clippy --workspace`).
- [ ] All relevant OpenSpec tasks in `tasks.md` are checked off `[x]`.
- [ ] Delta specs have been synchronized to `openspec/specs/` (if completing an OpenSpec change).
- [ ] Documentation (e.g., `README.md`, specs) has been updated accordingly.
