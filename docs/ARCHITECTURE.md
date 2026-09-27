# Architecture

## The big picture

```text
┌──────────────────────── ncpass (one desktop process) ────────────────────────┐
│                                                                              │
│   Svelte UI (src/)                       Rust core (src-tauri/src/)          │
│   runs in the OS web view                native code                         │
│                                                                              │
│   +page.svelte ── screen switcher        commands.rs ── functions the UI     │
│   Setup / Unlock / VaultView  ──invoke──▶                calls; holds state   │
│   lib/api.ts ── typed wrappers ◀─JSON──  api.rs ─────── HTTP to Nextcloud ───┼──▶ Nextcloud
│                                          vault.rs ───── encrypted file ──────┼──▶ vault.bin
└──────────────────────────────────────────────────────────────────────────────┘
```

A Tauri app has two halves:

- The **UI** is an ordinary web page (HTML/CSS/TypeScript, here with Svelte). It is shown
  by the operating system's built-in web view (WebKit on Linux/macOS, WebView2 on Windows).
  That's why Tauri apps are small: they don't ship a browser the way Electron does.
- The **core** is a Rust program. The UI calls it with `invoke("command_name", args)`;
  Tauri serialises the arguments to JSON, runs the matching `#[tauri::command]`
  function, and sends the result back as a JavaScript Promise.

All security-sensitive work is done in Rust: network, crypto, disk, clipboard.
The UI never receives the full list of passwords. It gets entries *without* the
secret and asks for one password at a time (`reveal` / `copy`).

## Files

| File | Role |
|---|---|
| `src-tauri/src/main.rs` | Program entry point; just calls `lib::run()` |
| `src-tauri/src/lib.rs` | Builds the Tauri app: plugins, shared state, list of commands |
| `src-tauri/src/api.rs` | Nextcloud Passwords API client; data types |
| `src-tauri/src/vault.rs` | Key derivation, encryption and decryption of `vault.bin`; unit test |
| `src-tauri/src/clipboard.rs` | Clipboard copy hidden from history + clear |
| `src-tauri/src/settings.rs` | Preferences in `settings.json` (auto-lock, clipboard clear); `get_settings`, `set_settings`; unit tests |
| `src-tauri/src/commands.rs` | `status`, `setup`, `unlock`, `lock`, `reset`, `get_vault`, `reveal`, `copy`, `sync` |
| `src-tauri/tauri.conf.json` | Window size, app id, CSP, bundle settings |
| `src-tauri/capabilities/default.json` | Which Tauri permissions the UI has |
| `src/routes/+page.svelte` | Picks the screen (setup / locked / open); global styles |
| `src/lib/api.ts` | TypeScript types and wrappers for each Rust command |
| `src/lib/Setup.svelte` | First-run form |
| `src/lib/Unlock.svelte` | Lock screen (and "forgot master password" reset) |
| `src/lib/VaultView.svelte` | Sidebar, search, list, detail, sync, idle auto-lock, keyboard shortcuts, version |
| `src/lib/Settings.svelte` | Settings dialog |

## Flows

**First run (`setup`)**
1. The user enters the server URL, Nextcloud user/password and a new master password.
2. Rust downloads passwords, folders and tags. This also proves the login works.
3. A random salt is generated, the key is derived from the master password, and
   `{credentials, snapshot}` is encrypted and written to `vault.bin`.

**Unlock**: read the salt from `vault.bin`, derive the key, decrypt. A wrong master password
fails the authentication check, so the app shows "Wrong master password".

**Sync**: runs automatically after unlocking and when you press ⟳. If the server
can't be reached, the error is shown ("Offline: using local copy") and the old data stays.
If it works, the new snapshot replaces the old one and the vault is re-saved.

**Lock**: happens manually or after 5 minutes idle. The in-memory session is dropped, and the
key is zeroed in memory (`Zeroizing`).

## Nextcloud API used

Base: `{server}/index.php/apps/passwords/api/1.0`, HTTP Basic auth, all `POST`:

| Endpoint | Body | Notes |
|---|---|---|
| `password/list` | `{"details":"model+tags"}` | tags come back as objects; we keep their ids |
| `folder/list` | `{"details":"model"}` | root folder id is `00000000-0000-0000-0000-000000000000` |
| `tag/list` | `{"details":"model"}` | |

Trashed items are skipped. Client-side encryption (CSE) is **not** supported. It's not
enabled on this account, so every field arrives as plain text over HTTPS.

## Vault file format (`vault.bin`)

```text
| "NCPV1" (5 bytes) | salt (16 bytes) | nonce (24 bytes) | ciphertext + 16-byte tag |
```

- **Key derivation**: Argon2id, 64 MiB memory, 3 iterations, 1 lane → 32-byte key.
  This is deliberately slow, which makes brute-forcing a stolen `vault.bin` expensive.
- **Encryption**: XChaCha20-Poly1305 (authenticated). Every save uses a new random nonce.
- **Plaintext**: JSON of `VaultData { credentials, snapshot }`.
- Saving writes `vault.tmp` and then renames it, so a crash can't leave a half-written vault.

## Security model and known limits

- ✅ The data on disk is useless without the master password.
- ✅ Copied values are flagged "don't keep in history" (KDE Klipper hint on Linux,
  Win+V history on Windows, concealed type on macOS) and wiped after 20 s if unchanged.
  See `src-tauri/src/clipboard.rs` for why the flag is also needed for clearing to work on KDE.
- ✅ The UI only holds one revealed password at a time.
- ✅ A strict CSP stops the web view from loading remote content.
- ⚠️ The Nextcloud account password is stored inside the vault, because app passwords
  are not available right now. Switch to an app password when you can: it can be revoked
  per device.
- ⚠️ While unlocked, decrypted data is in the Rust process memory. That's normal for
  password managers.
- ⚠️ No protection against malware running as your user (keyloggers, etc.), and no
  password manager can provide that.
- ⚠️ The master password strength is up to you. Use a long passphrase.
