# AGENTS.md — ltspice-renderer

## Dev environment

- **Nix + devenv** — run `direnv allow` then `devenv shell` to enter.
- Tools in the shell: `cargo`, `clippy`, `rustfmt`, `rust-analyzer`, `cargo-llvm-cov`, `cargo-nextest`, `ltspice` (Wine).
- Unfree packages are allowed (see `devenv.yaml` `allow_unfree: true`).

## Build & test

```bash
cargo build
cargo test                  # runs from workspace root, uses test_files/
devenv shell cargo test     # how CI runs it
cargo nextest run           # faster alternative (available in nix shell)
cargo llvm-cov              # coverage (available in nix shell)
cargo clippy                # lint
cargo fmt                   # formatting
```

## Architecture

Single Rust binary crate. Entrypoint: `src/main.rs`.

| Module | Purpose |
|---|---|
| `schematic.rs` | Parses `.asc` schematic files |
| `symbol.rs` / `symbol_loader.rs` | Parses `.asy` symbol files, loads them from local LTspice lib |
| `svg_renderer.rs` | Renders parsed schematics/symbols to SVG |
| `shape.rs` | Geometry types (Point, Line, Rectangle, Circle, Text) |
| `bounding_box.rs` | Bounding box computation |
| `file_reader.rs` | File I/O with UTF-8/UTF-16LE auto-detection |

## Notable details

- All modules use `#![allow(dead_code)]` — intentional, many types/APIs are work-in-progress.
- LTspice library symbols are loaded from local paths (Windows AppData or Linux Wine paths) — see `symbol_loader.rs:default_library_paths()`.
- Test `load_all_available_library_symbols` requires a local LTspice installation (fails in CI).
- Test `test_all_asc_files_from_examples` reads LTspice's bundled examples — also requires local install.
- `.svg` and `.txt` outputs are gitignored.
- `fetch-ltspice-lib.sh` downloads and extracts the LTspice library zip from Analog Devices' MSI installer (uses `nix-shell -p curl msitools unzip`).
- Spell-check word lists in `.vscode/settings.json` and `codebook.toml` include LTspice jargon (`PINATTR`, `SYMATTR`, `IOPIN`, `asy`, `wineprefixes`, etc.).