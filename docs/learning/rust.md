# Learning Rust with ncpass

This guide explains the Rust features the project uses, each with an example from our
own code. Read it next to the source files. Good external resources:

- [The Rust Book](https://doc.rust-lang.org/book/) is the official guide, free and excellent.
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/)
- [Rustlings](https://github.com/rust-lang/rustlings) are small exercises you fix until they compile.

---

## 1. Project layout and Cargo

`src-tauri/Cargo.toml` is the manifest. It plays the role `package.json` plays for JS.
`[dependencies]` lists **crates** (libraries) from <https://crates.io>:

| Crate | Why we use it |
|---|---|
| `tauri` | the desktop app framework |
| `serde`, `serde_json` | converting structs ⇄ JSON |
| `reqwest` | HTTP client |
| `tokio` | async runtime (timers, running things concurrently) |
| `argon2` | password → key derivation |
| `chacha20poly1305` | encryption |
| `zeroize` | wiping secrets from memory |

`cargo add <crate>` adds one. `cargo build`, `cargo test`, `cargo clippy` do what they say.

**Modules**: each `.rs` file is a module. `lib.rs` declares them:

```rust
mod api;       // loads src/api.rs
mod commands;
mod vault;
```

Then `use crate::api::Credentials;` imports a name from another module. Items are
private by default, and `pub` makes them visible outside the module.

## 2. Variables, mutability, types

```rust
let salt = vault::new_salt();          // immutable (the default)
let mut salt = [0u8; SALT_LEN];        // `mut` = can be changed
```

Rust infers most types. `[0u8; 16]` is an array of 16 bytes, all zero. `u8` is an unsigned
8-bit int, `i64` a signed 64-bit int, `String` owned text, `&str` borrowed text,
`Vec<T>` a growable list.

## 3. Structs and derive

```rust
#[derive(Clone, Serialize, Deserialize)]
pub struct Credentials {
    pub server: String,
    pub user: String,
    pub password: String,
}
```

`#[derive(...)]` asks the compiler to generate code: `Clone` adds `.clone()`, and
`Serialize`/`Deserialize` (from serde) add conversion to and from JSON.
`#[serde(rename_all = "camelCase")]` maps `synced_at` in Rust to `syncedAt` in JSON,
because each language has its own naming convention.

`#[serde(default)]` on a field (see `ApiPassword` in `api.rs`) means "if the JSON doesn't
have it, use the default (empty string, `false`, 0…)". This makes parsing forgiving.

## 4. Ownership and borrowing ⭐ (the core idea of Rust)

Every value has exactly **one owner**. When the owner goes out of scope, the value is freed.
There's no garbage collector, and you never call `free`.

- Passing a value **moves** it: the old variable can't be used any more.
- `&x` **borrows** it read-only, and `&mut x` borrows it for writing. At any moment there
  can be many `&` or exactly one `&mut`, never both. This rule is what prevents data races.
- `.clone()` makes an independent copy when you really need two owners.

In `api.rs`, `fetch_all` uses `into_iter()`, which **consumes** the downloaded `Vec` and moves
each field into our own struct without copying:

```rust
pw.into_iter().map(|p| Password { id: p.id, label: p.label, /* ... */ })
```

In `commands.rs`, `get_vault` only has a borrow of the session, so it must `clone()` the
strings it returns:

```rust
label: p.label.clone(),
```

`vault::save(path: &Path, key: &Key, ...)` takes borrows: it only needs to *look at* them.

## 5. Option and Result: no null, no exceptions

```rust
enum Option<T> { Some(T), None }        // "maybe a value"
enum Result<T, E> { Ok(T), Err(E) }     // "a value or an error"
```

Our session is `Mutex<Option<Session>>`, and `None` means locked.
Every fallible function returns `Result<_, String>` (we use plain strings as errors to keep
it simple).

The **`?` operator** says "if this is an `Err`, return it from my function now, otherwise
unwrap the `Ok` value":

```rust
let bytes = fs::read(path).map_err(|e| e.to_string())?;
```

`map_err` converts the library's error type into our `String`. `ok_or("Vault is locked")?`
turns an `Option` into a `Result`.

In `setup` you'll see `??`. `spawn_blocking(...).await` returns
`Result<Result<Session, String>, JoinError>`: the first `?` handles "the thread crashed"
and the second handles our own error.

`match` handles every case explicitly:

```rust
match resp.status().as_u16() {
    200 => /* parse */,
    401 => Err("Login failed: check user and password".into()),
    code => Err(format!("Server replied with HTTP {code}")),
}
```

## 6. Closures and iterators

`|x| expr` is an anonymous function, the equivalent of JS's `x => expr`. Iterators are
chains like JS array methods, but lazy and zero-cost:

```rust
p.tags.into_iter().map(|t| t.id).collect()
fo.into_iter().filter(|f| !f.trashed).map(...).collect()
```

`collect()` builds the result collection. Rust figures out which kind from the expected type.

## 7. Generics and traits

```rust
async fn list<T: for<'de> Deserialize<'de>>(&self, endpoint: &str, details: &str) -> Result<T, String>
```

One function works for `Vec<ApiPassword>`, `Vec<ApiFolder>`, and so on: any `T` that serde
can deserialise. A **trait** is like an interface (`Serialize`, `Clone`, …). Ignore the
`for<'de>` part for now: it's a lifetime detail serde needs.

`impl Client { ... }` adds methods to a struct. `&self` is like `this`, borrowed.

## 8. async / await

Network calls are `async`: they return a *future*, and `.await` waits for it without
blocking the thread. `tokio::try_join!` runs our three downloads concurrently:

```rust
let (pw, fo, ta) = tokio::try_join!(
    self.list::<Vec<ApiPassword>>("password/list", "model+tags"),
    self.list::<Vec<ApiFolder>>("folder/list", "model"),
    self.list::<Vec<ApiTag>>("tag/list", "model"),
)?;
```

CPU-heavy work (Argon2) goes into `spawn_blocking` so it doesn't stall other async tasks.

## 9. Shared state: Mutex

Tauri may run commands on several threads at once, so shared state must be thread-safe.
`Mutex<T>` allows only one thread in at a time:

```rust
let guard = state.0.lock().unwrap();   // wait for our turn
// ... use *guard ...
                                        // guard dropped at end of scope → unlocked
```

In `sync` we copy the credentials and **release the lock before** the slow network call,
then lock again to store the result. Holding a lock across `.await` is a classic mistake.

`AppState(pub Mutex<...>)` is a *tuple struct*: a struct with one unnamed field,
accessed as `.0`.

## 10. Tauri specifics

```rust
#[tauri::command]
pub fn reveal(state: State<AppState>, id: String) -> Result<String, String>
```

- `#[tauri::command]` makes the function callable from JS: `invoke("reveal", { id })`.
- Parameters like `AppHandle` and `State<T>` are **injected** by Tauri. The others come
  from the JS arguments (by name).
- Returning `Err(msg)` makes the JS promise reject with `msg`.
- Each command must be listed in `generate_handler![...]` in `lib.rs`.
- Async commands that borrow state need a lifetime: `State<'_, AppState>`.

## 11. Crypto concepts used

- **Salt**: random bytes stored in the clear. They make the same master password
  produce different keys on different installs, which defeats precomputed tables.
- **KDF (Argon2id)**: turns a human password into a 32-byte key, slowly and using lots of
  memory on purpose, so each guess is expensive for an attacker.
- **AEAD (XChaCha20-Poly1305)**: encrypts *and* authenticates. Decryption fails if
  the key is wrong or a single byte was changed. That's how we detect a wrong master password.
- **Nonce**: a "number used once". Never reuse a nonce with the same key, so we generate a
  random one on every save. The 24-byte XChaCha nonce is long enough for random ones to be safe.
- **Zeroizing**: a wrapper that overwrites memory with zeros when dropped.

## 12. Tests

In `vault.rs`:

```rust
#[cfg(test)]           // only compiled for `cargo test`
mod tests {
    #[test]
    fn roundtrip_and_wrong_password() { ... assert!(load(&path, &wrong).is_err()); }
}
```

Run with `cd src-tauri && cargo test`.

## Exercises (in increasing difficulty)

1. Change `CLIPBOARD_CLEAR_SECS` to 30 and verify.
2. Add a `copy` option for `"notes"` in `field()` (`commands.rs`) and a button in the UI.
3. Write a test in `vault.rs` that corrupts one byte of the file and checks `load` fails.
4. Replace `String` errors with an `enum Error` using the `thiserror` crate.
5. Add a `change_master(old, new)` command: unlock with old, new salt + key, save.
