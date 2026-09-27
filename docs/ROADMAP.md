# Roadmap

## Phase 1: read-only offline client ✅

- [x] Connect with Nextcloud user/password
- [x] Download passwords, folders, tags
- [x] Encrypted local vault (Argon2id + XChaCha20-Poly1305)
- [x] Folder tree, tags, favorites, search
- [x] Reveal/copy with clipboard auto-clear
- [x] Sync on unlock and on demand; works offline
- [x] Auto-lock when idle (5 min by default, configurable)
- [x] Tested on Linux (KDE Wayland), including clipboard clearing
- [x] Tested on macOS 12 and Windows (v0.1.2)

## Phase 1.5: polish ✅

- [x] GitHub Actions workflow that builds installers for all 3 OSes on each tag
- [x] App icon (source: `src-tauri/icons/source.svg`)
- [x] Keyboard shortcuts (Ctrl+F search, Ctrl+C copy password, Ctrl+L lock, Esc clears search)
- [x] Settings: auto-lock time, clipboard clear time
- [x] Use a Nextcloud app password (setup screen suggests one)

## Phase 2: editing (next)

The API uses a `revision`/`hash` per item. Updates must send the current revision,
which lets us detect conflicts.

- [ ] Create / edit / delete (move to trash) while **online only**
- [ ] Password generator (the API has `service/password`)
- [ ] Maybe: queue offline edits and replay them on the next sync, with conflict
      detection. This is the "hard" part. Only do it if it proves necessary.
