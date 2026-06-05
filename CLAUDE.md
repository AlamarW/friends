# friends — developer guide

## Project overview

Terminal-based smart address book. Friends have typed profile fields; filling in fields unlocks **applets** — small context-aware tools per friend (job feed, book recs, conversation prep). All APIs are keyless and free.

## Structure

```
src/
  main.rs          — tokio::main, terminal setup, event loop, async fetch dispatch
  app.rs           — AppState, Screen enum, EditState (form logic), LoadState<T>
  ui.rs            — All ratatui rendering (one fn per screen, no logic)
  events.rs        — Keyboard → AppState mutations; returns EventAction for async work
  apps/
    mod.rs         — AppletKind enum, available_applets(friend) unlock logic
  data/
    friend.rs      — Friend, JobHuntProfile, BookProfile structs + serde
    storage.rs     — load_from(path) / save_to(friends, path); load_friends/save_friends wrap these
  api/
    remotive.rs    — fetch_jobs(profile)     → Vec<Job>      (Remotive API)
    open_library.rs— fetch_books(profile)    → Vec<Book>     (Open Library API)
    wikipedia.rs   — fetch_summaries(topics) → Vec<WikiSummary> (Wikipedia REST)
  tests.rs         — Integration-style tests (included via #[cfg(test)] mod tests in main.rs)
```

## Key design rules

**Keep files small.** If a file grows past ~200 lines, split it. `ui.rs` is the only justified exception (many small render fns, no logic). `main.rs` should stay thin — all logic belongs in `app.rs`, `events.rs`, or `apps/`.

**Adding a new applet** requires exactly three changes:
1. New optional profile struct on `Friend` in `data/friend.rs`
2. New `AppletKind` variant + unlock condition in `apps/mod.rs`
3. New API module in `api/`, new render fn in `ui.rs`, new `FetchResult` variant in `main.rs`

**No logic in `ui.rs`.** Rendering only. All state mutation happens in `events.rs`.

**Storage is path-based.** Use `load_from(path)` / `save_to(friends, path)` in tests (temp dirs). `load_friends()` / `save_friends()` are convenience wrappers for production use only.

## Testing conventions

Target: **≥ 85% coverage** on all non-UI, non-main code.

### Where tests live

| File | What to test |
|------|-------------|
| `data/friend.rs` | Struct defaults, TOML roundtrips, serde edge cases |
| `data/storage.rs` | `load_from` / `save_to` with `TempDir`; error paths |
| `apps/mod.rs` | Every unlock condition branch for each applet |
| `app.rs` | `parse_csv`, `EditState` construction/mutation/apply, `AppState` accessors |
| `events.rs` | Every `handle_*` fn: screen transitions, field edits, action returns |
| `src/tests.rs` | End-to-end cycles: form → friend → applets; storage roundtrip; `AppState` integration |

### Rules

- **Unit tests** go in `#[cfg(test)] mod tests` at the bottom of the relevant file.
- **Integration-style tests** (multi-module flows, storage + state together) go in `src/tests.rs`.
- **Live API tests** go in `src/tests.rs` marked `#[ignore]`. Run them with `cargo test -- --ignored`.
- Use `TempDir` (from `tempfile`) for any test that writes to disk — never write to `~/.friends/` in tests.
- Test names follow `test_<what>_<condition_or_result>` (e.g. `test_job_feed_does_not_unlock_with_empty_role`).
- One assertion per logical claim. Prefer specific asserts over `assert!(result.is_ok())` where possible.

### Running tests

```bash
cargo test                          # all unit + integration (skips live API)
cargo test -- --ignored             # live API tests only (needs network)
cargo test data::storage            # one module
cargo test test_job_feed            # by name pattern
```

## Data file

Platform-specific location via `dirs::data_dir()`:
- **Linux:** `~/.local/share/friends/friends.toml`
- **macOS:** `~/Library/Application Support/friends/friends.toml`
- **Windows:** `%APPDATA%\friends\friends.toml`

Human-editable TOML. Hand-editing is a supported workflow; the schema should remain readable without documentation.
