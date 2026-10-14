## Purpose

Generates a standalone macOS `Spoty.app` application bundle and continuous integration pipeline for building and testing across target platforms.

## ADDED Requirements

### Requirement: Standalone macOS App Bundle Generation
The packaging pipeline SHALL generate a valid macOS application bundle (`Spoty.app`) containing the compiled release binary, `Info.plist`, and `PkgInfo`.

#### Scenario: Running macOS packaging script
- **WHEN** `scripts/package-macos.sh` is executed on macOS
- **THEN** a `target/dist/Spoty.app` bundle is generated with `Contents/MacOS/spoty` and valid `Info.plist` metadata

### Requirement: Continuous Integration Pipeline
The repository SHALL include a GitHub Actions CI workflow verifying `cargo test --workspace` on macOS and Linux.

#### Scenario: Automated PR testing
- **WHEN** changes are pushed to `main`
- **THEN** the CI workflow runs the full test suite and verifies clean compilation
