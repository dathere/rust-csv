# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**rust-csv** is the standard CSV library for Rust (crate name: `csv`), originally by BurntSushi. It provides fast CSV parsing and writing with full Serde support. This is a Cargo workspace with three crates:

- **csv** (root) — High-level reader/writer with Serde integration. Edition 2021, MSRV 1.93.
- **csv-core** (`csv-core/`) — Low-level `no_std` DFA-based parser. Edition 2018.
- **csv-index** (`csv-index/`) — On-disk CSV indexing for random access. Edition 2018.

## Build & Test Commands

```bash
cargo build --verbose                              # Build main crate
cargo test --verbose                               # Test main crate
cargo test --verbose --manifest-path csv-core/Cargo.toml  # Test csv-core
cargo test --verbose --manifest-path csv-index/Cargo.toml # Test csv-index

# Run a single test
cargo test test_name -- --nocapture

# Formatting (enforced in CI)
cargo fmt --all --check

# Miri memory safety (CI runs this)
cargo miri test --lib --verbose
cargo miri test --doc --verbose

# Benchmarks (nightly only)
cargo bench --verbose --no-run
```

## Code Style

- **rustfmt**: `max_width=79`, `use_small_heuristics="max"` (see `rustfmt.toml`)
- **Docs enforced**: `#![deny(missing_docs)]` — all public items must have doc comments
- Run `cargo fmt --all` before committing

## Architecture

**Layered design**: `csv-core` handles raw byte-level parsing (DFA-based, no allocations, `no_std`). The `csv` crate wraps it with `Reader`/`Writer` types, `StringRecord`/`ByteRecord`, and Serde support.

Key source files in `src/`:
- `reader.rs` / `writer.rs` — `Reader<R>`/`Writer<W>` and their builders
- `byte_record.rs` / `string_record.rs` — Record types (ByteRecord for raw bytes, StringRecord for UTF-8)
- `deserializer.rs` / `serializer.rs` — Custom Serde implementations
- `tutorial.rs` / `cookbook.rs` — Documentation modules containing examples that are synced to `examples/`

## CI Checks

CI tests across: pinned MSRV (1.93), stable, beta, nightly, macOS, Windows (MSVC + GNU). Additional jobs:
- `cargo fmt --all --check`
- `ci/check-copy cookbook` and `ci/check-copy tutorial` — verifies example files in `examples/` are in sync with `src/tutorial.rs` and `src/cookbook.rs`
- Miri with `-Zmiri-strict-provenance`

## Example Sync

Tutorial and cookbook examples live in `src/tutorial.rs` and `src/cookbook.rs` as doc comments. They are copied to `examples/` via `scripts/copy-examples`. CI verifies they stay in sync — if you edit examples, edit the source modules and run the sync script.

## Features

- `simd_utf8_compat` — Uses simdutf8 compat mode (slower but returns error position info, ~16x faster than std). Default off uses basic mode (~23x faster than std).
