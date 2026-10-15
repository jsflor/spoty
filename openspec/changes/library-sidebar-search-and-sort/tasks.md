# Tasks: library-sidebar-search-and-sort

## 1. Sidebar Search & Sort State (`crates/ui/src/app.rs`)

- [ ] 1.1 Add `LibrarySortOrder` enum and state fields (`sidebar_search_query`, `sidebar_sort_order`) to `SpotyApp`.
- [ ] 1.2 Implement sidebar search input box and sort dropdown menu in `crates/ui/src/app.rs`.

## 2. Dynamic Filtering & Sorting Implementation

- [ ] 2.1 Apply real-time string filtering against playlist titles and artist names.
- [ ] 2.2 Apply selected sort order ("Recents", "Alphabetical", "Recently Added") prior to rendering list items.
- [ ] 2.3 Verify clean compilation with `cargo check -p spoty-ui`.

## 3. Verification and Quality Gates

- [ ] 3.1 Write unit tests for library sidebar sorting logic.
- [ ] 3.2 Run full workspace test suite `cargo test --workspace`.
- [ ] 3.3 Verify formatting and compiler checks with `cargo fmt --all -- --check` and `cargo check --workspace`.
