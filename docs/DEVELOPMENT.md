# Development

## Prerequisites

All platforms: **Rust** (via [rustup](https://rustup.rs)), **Node.js** 20+ and **pnpm**.

### Linux (Ubuntu/Debian)

```bash
sudo apt install libwebkit2gtk-4.1-dev build-essential curl wget file \
  libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev
```

### macOS

```bash
xcode-select --install
```

### Windows

Install the "Microsoft C++ Build Tools" (Desktop development with C++) and make sure
WebView2 is present (it is on up-to-date Windows 10/11).

Full details: <https://tauri.app/start/prerequisites/>

## Everyday commands

| Command | What it does |
|---|---|
| `pnpm install` | Install JS dependencies |
| `pnpm tauri dev` | Run the app with hot reload (UI changes apply instantly; Rust changes rebuild) |
| `pnpm check` | Type-check the Svelte/TypeScript code |
| `cd src-tauri && cargo test` | Run the Rust unit tests |
| `cd src-tauri && cargo clippy` | Rust linter (catches common mistakes) |
| `cd src-tauri && cargo test -- --ignored --nocapture` | Opt-in checks against the real server (`live`, needs `NEXTCLOUD_INSTANCE`, `NEXTCLOUD_USER`, `NEXTCLOUD_PASSWORD` env vars) and desktop clipboard (`clipboard`) |
| `pnpm tauri build` | Build an installer for **the current OS** into `src-tauri/target/release/bundle/` |

## Cargo cheat sheet

Cargo is Rust's build tool and package manager. Run these inside `src-tauri/`,
where `Cargo.toml` lives. For day-to-day app work, `pnpm tauri dev` and
`pnpm tauri build` call Cargo for you; the commands below are for working on
the Rust side directly.

### Build and check

| Command | What it does |
|---|---|
| `cargo check` | Type-checks without producing a binary. It's the fastest way to see compile errors, so use it often |
| `cargo build` | Debug build into `target/debug/`. It compiles fast, but the result runs slowly |
| `cargo build --release` | Optimized build into `target/release/`. Slower to compile, fast to run |
| `cargo run` | Build and run the binary (for the app, prefer `pnpm tauri dev` so the UI is served too) |

### Test and lint

| Command | What it does |
|---|---|
| `cargo test` | Run all unit tests |
| `cargo test vault` | Run only tests whose name contains `vault` |
| `cargo test -- --nocapture` | Show `println!` output from tests (hidden by default) |
| `cargo test -- --ignored` | Run only the `#[ignore]` tests (the live server and clipboard checks) |
| `cargo clippy` | Linter with hundreds of checks for common mistakes and non-idiomatic code |
| `cargo clippy --fix` | Apply clippy's suggested fixes automatically |
| `cargo fmt` | Format all code in the standard style (`cargo fmt --check` only reports) |

### Dependencies

| Command | What it does |
|---|---|
| `cargo add serde --features derive` | Add a crate to `Cargo.toml` (use `cargo remove <crate>` to drop one) |
| `cargo update` | Upgrade dependencies within the versions `Cargo.toml` allows; this rewrites `Cargo.lock` |
| `cargo tree` | Show the dependency tree (`cargo tree -i <crate>` shows who pulls a crate in) |
| `cargo doc --open` | Build HTML docs for this crate and all dependencies, then open them in the browser |

### Cache and disk space

Everything Cargo compiles goes into `src-tauri/target/`. That folder gets
large (10+ GiB is normal for a Tauri app) and it is safe to delete, because it
is fully rebuildable.

| Command | What it does |
|---|---|
| `cargo clean` | Delete the whole `target/` folder. The next build is slow (a full rebuild) |
| `cargo clean --release` | Delete only the release build output |
| `cargo clean -p ncpass` | Delete only this crate's build output and keep the dependencies (the quick fix for odd stale-build errors) |

When to clean: after renaming or moving the project folder, after a Rust
toolchain update if builds act strangely, or to free disk space. The
downloaded crate sources live separately in `~/.cargo/registry/` and are
shared by all projects.

### Toolchain (rustup)

| Command | What it does |
|---|---|
| `rustup update` | Update Rust itself (compiler, cargo, clippy, rustfmt) |
| `rustc --version` | Show the installed compiler version |
| `rustup doc --book` | Open *The Rust Programming Language* book offline |

## Building for each OS

Tauri can't cross-compile installers reliably, so each OS builds its own:

- **Linux**: `pnpm tauri build` makes `.deb`, `.rpm` and `.AppImage`.
- **macOS** (e.g. on `newpath.local`): same command, makes `.app` / `.dmg`.
  The app isn't signed, so the first launch needs right-click → Open.
- **Windows**: same command, makes `.msi` / `.exe`.

Or let GitHub build all three: see [RELEASING.md](RELEASING.md).

## Where the data lives

The encrypted vault is `vault.bin` in the app data directory:

| OS | Path |
|---|---|
| Linux | `~/.local/share/website.vitorcorreia.ncpass/` |
| macOS | `~/Library/Application Support/website.vitorcorreia.ncpass/` |
| Windows | `%APPDATA%\website.vitorcorreia.ncpass\` |

Deleting it just means you set up again; nothing on the server is affected.
