# ncpass

A small desktop client for [Nextcloud Passwords](https://github.com/marius-wieschollek/passwords)
that keeps an **encrypted offline copy** of your vault, so your passwords stay usable
when the Nextcloud server can't be reached.

Built with [Tauri 2](https://tauri.app) (Rust backend) and [Svelte 5](https://svelte.dev) (UI).
Runs on Linux, macOS 10.15+ and Windows 10/11.

## Status

**Read-only (phases 1 and 1.5 done).** Sync, browse folders/tags, search, reveal/copy,
auto-lock, configurable timings, keyboard shortcuts (`Ctrl+F` search, `Ctrl+C` copy password,
`Ctrl+L` lock; `Cmd` on macOS). Editing is planned for phase 2 (see [docs/ROADMAP.md](docs/ROADMAP.md)).

## Documentation

| Doc | What's in it |
|---|---|
| [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) | How the app is built, the vault file format, the security model |
| [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md) | Installing prerequisites, running, building installers |
| [docs/RELEASING.md](docs/RELEASING.md) | Tags, releases, building installers for all OSes |
| [docs/ROADMAP.md](docs/ROADMAP.md) | Phases and ideas |
| [docs/learning/rust.md](docs/learning/rust.md) | Rust concepts, explained using this project's code |
| [docs/learning/svelte.md](docs/learning/svelte.md) | Svelte + TypeScript concepts, explained using this project's code |

## Quick start (development)

```bash
pnpm install
pnpm tauri dev
```

The first time the app starts, it asks for your Nextcloud URL, user and an
**app password** (Nextcloud: Personal settings → Security → Devices & sessions), plus a
**master password** that encrypts the local copy. That master password cannot be recovered.

## Credits

Built by [Vítor Correia](https://github.com/vitorhmcorreia), pair-programming with
[Claude Code](https://claude.com/claude-code) (Anthropic's Claude Opus 5.5), which wrote
much of the code and the learning guides in `docs/learning/` and is credited as co-author
on the commits.

Thanks to [Marius Wieschollek](https://github.com/marius-wieschollek) for
[Nextcloud Passwords](https://github.com/marius-wieschollek/passwords), whose API this app uses.

## License

[MIT](LICENSE) © 2026 Vítor Correia
