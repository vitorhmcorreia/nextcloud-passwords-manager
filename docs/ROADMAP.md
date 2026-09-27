# Roadmap

## Phase 1: read-only offline client ✅ (current)

- [x] Connect with Nextcloud user/password
- [x] Download passwords, folders, tags
- [x] Encrypted local vault (Argon2id + XChaCha20-Poly1305)
- [x] Folder tree, tags, favorites, search
- [x] Reveal/copy with clipboard auto-clear
- [x] Sync on unlock and on demand; works offline
- [x] Auto-lock after 5 min idle
- [x] Tested on Linux (KDE Wayland), including clipboard clearing
- [ ] Try it on macOS 12 and Windows

## Phase 1.5: polish

- [x] GitHub Actions workflow that builds installers for all 3 OSes on each tag
- [ ] App icon
- [ ] Keyboard shortcuts (Ctrl+F search, Ctrl+C copy password, Ctrl+L lock)
- [ ] TOTP codes from custom fields (if used)
- [ ] Settings: auto-lock time, clipboard clear time
- [ ] Switch to a Nextcloud app password when available

## Phase 2: editing

The API uses a `revision`/`hash` per item. Updates must send the current revision,
which lets us detect conflicts.

- [ ] Create / edit / delete (move to trash) while **online only**
- [ ] Password generator (the API has `service/password`)
- [ ] Maybe: queue offline edits and replay them on the next sync, with conflict
      detection. This is the "hard" part. Only do it if it proves necessary.
