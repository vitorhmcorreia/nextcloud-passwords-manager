# ncpass

A small desktop client for [Nextcloud Passwords](https://github.com/marius-wieschollek/passwords)
that keeps an **encrypted offline copy** of your vault, so your passwords stay usable
when the Nextcloud server can't be reached.

Built with [Tauri 2](https://tauri.app) (Rust backend) and [Svelte 5](https://svelte.dev) (UI).
Runs on Linux, macOS 10.15+ and Windows 10/11.

## Status

**Phase 1: read-only.** Sync, browse folders/tags, search, reveal/copy, auto-lock.
Editing is planned for phase 2 (see [docs/ROADMAP.md](docs/ROADMAP.md)).

## Documentation

| Doc | What's in it |
|---|---|
| [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) | How the app is built, the vault file format, the security model |
| [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md) | Installing prerequisites, running, building installers |
| [docs/ROADMAP.md](docs/ROADMAP.md) | Phases and ideas |
| [docs/learning/rust.md](docs/learning/rust.md) | Rust concepts, explained using this project's code |
| [docs/learning/svelte.md](docs/learning/svelte.md) | Svelte + TypeScript concepts, explained using this project's code |

## Quick start (development)

```bash
pnpm install
pnpm tauri dev
```

The first time the app starts, it asks for your Nextcloud URL, user and password, plus a
**master password** that encrypts the local copy. That master password cannot be recovered.
