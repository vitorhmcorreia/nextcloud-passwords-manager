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
| `cd src-tauri && cargo test -- --ignored --nocapture` | Opt-in checks against the real server (`live`, needs `.env` loaded) and desktop clipboard (`clipboard`) |
| `pnpm tauri build` | Build an installer for **the current OS** into `src-tauri/target/release/bundle/` |

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
