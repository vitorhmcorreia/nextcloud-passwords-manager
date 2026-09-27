//! Functions the Svelte frontend can call with `invoke("name", {...})`.
//!
//! Secrets stay in Rust as much as possible: the list sent to the UI has no
//! passwords; the UI asks for one explicitly with `reveal` or `copy`.

use crate::api::{Client, Credentials, Folder, Tag};
use crate::vault::{self, Key, VaultData};
use serde::Serialize;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::Duration;
use tauri::{AppHandle, Manager, State};
use tauri_plugin_clipboard_manager::ClipboardExt;

/// Seconds before a copied secret is wiped from the clipboard.
const CLIPBOARD_CLEAR_SECS: u64 = 20;

/// An unlocked vault held in memory.
pub struct Session {
    key: Key,
    salt: [u8; 16],
    data: VaultData,
}

/// Shared app state. `None` means locked.
#[derive(Default)]
pub struct AppState(pub Mutex<Option<Session>>);

fn vault_path(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?
        .join("vault.bin"))
}

#[derive(Serialize)]
pub struct Status {
    exists: bool,
    unlocked: bool,
}

#[tauri::command]
pub fn status(app: AppHandle, state: State<AppState>) -> Result<Status, String> {
    Ok(Status {
        exists: vault_path(&app)?.exists(),
        unlocked: state.0.lock().unwrap().is_some(),
    })
}

/// First run: check the credentials work, download everything, create the vault.
#[tauri::command]
pub async fn setup(
    app: AppHandle,
    state: State<'_, AppState>,
    server: String,
    user: String,
    password: String,
    master: String,
) -> Result<(), String> {
    if master.chars().count() < 8 {
        return Err("Master password must have at least 8 characters".into());
    }
    let credentials = Credentials { server: server.trim().to_string(), user, password };
    let snapshot = Client::new(credentials.clone())?.fetch_all().await?;

    let path = vault_path(&app)?;
    // Argon2 takes a moment of CPU; run it off the async worker threads.
    let session = tauri::async_runtime::spawn_blocking(move || -> Result<Session, String> {
        let salt = vault::new_salt();
        let key = vault::derive_key(&master, &salt)?;
        let data = VaultData { credentials, snapshot: Some(snapshot) };
        vault::save(&path, &key, &salt, &data)?;
        Ok(Session { key, salt, data })
    })
    .await
    .map_err(|e| e.to_string())??;

    *state.0.lock().unwrap() = Some(session);
    Ok(())
}

#[tauri::command]
pub async fn unlock(app: AppHandle, state: State<'_, AppState>, master: String) -> Result<(), String> {
    let path = vault_path(&app)?;
    let session = tauri::async_runtime::spawn_blocking(move || -> Result<Session, String> {
        let salt = vault::read_salt(&path)?;
        let key = vault::derive_key(&master, &salt)?;
        let data = vault::load(&path, &key)?;
        Ok(Session { key, salt, data })
    })
    .await
    .map_err(|e| e.to_string())??;

    *state.0.lock().unwrap() = Some(session);
    Ok(())
}

#[tauri::command]
pub fn lock(state: State<AppState>) {
    // Dropping the session wipes the key (Zeroizing) and frees the data.
    *state.0.lock().unwrap() = None;
}

/// Delete the local vault (e.g. forgotten master password). Server data is untouched.
#[tauri::command]
pub fn reset(app: AppHandle, state: State<AppState>) -> Result<(), String> {
    *state.0.lock().unwrap() = None;
    let path = vault_path(&app)?;
    if path.exists() {
        std::fs::remove_file(path).map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// A password entry without the secret itself.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EntryView {
    id: String,
    label: String,
    username: String,
    url: String,
    notes: String,
    folder: String,
    tags: Vec<String>,
    favorite: bool,
    edited: i64,
    custom_fields: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VaultView {
    server: String,
    user: String,
    synced_at: Option<i64>,
    entries: Vec<EntryView>,
    folders: Vec<Folder>,
    tags: Vec<Tag>,
}

#[tauri::command]
pub fn get_vault(state: State<AppState>) -> Result<VaultView, String> {
    let guard = state.0.lock().unwrap();
    let s = guard.as_ref().ok_or("Vault is locked")?;
    let creds = &s.data.credentials;
    let snap = s.data.snapshot.as_ref();
    Ok(VaultView {
        server: creds.server.clone(),
        user: creds.user.clone(),
        synced_at: snap.map(|x| x.synced_at),
        entries: snap
            .map(|x| {
                x.passwords
                    .iter()
                    .map(|p| EntryView {
                        id: p.id.clone(),
                        label: p.label.clone(),
                        username: p.username.clone(),
                        url: p.url.clone(),
                        notes: p.notes.clone(),
                        folder: p.folder.clone(),
                        tags: p.tags.clone(),
                        favorite: p.favorite,
                        edited: p.edited,
                        custom_fields: p.custom_fields.clone(),
                    })
                    .collect()
            })
            .unwrap_or_default(),
        folders: snap.map(|x| x.folders.clone()).unwrap_or_default(),
        tags: snap.map(|x| x.tags.clone()).unwrap_or_default(),
    })
}

/// Look up one field of one entry.
fn field(state: &AppState, id: &str, which: &str) -> Result<String, String> {
    let guard = state.0.lock().unwrap();
    let s = guard.as_ref().ok_or("Vault is locked")?;
    let p = s
        .data
        .snapshot
        .as_ref()
        .and_then(|x| x.passwords.iter().find(|p| p.id == id))
        .ok_or("Entry not found")?;
    Ok(match which {
        "password" => p.password.clone(),
        "username" => p.username.clone(),
        "url" => p.url.clone(),
        _ => return Err(format!("Unknown field {which}")),
    })
}

#[tauri::command]
pub fn reveal(state: State<AppState>, id: String) -> Result<String, String> {
    field(&state, &id, "password")
}

/// Copy a field to the clipboard, then clear it after a delay
/// (only if the clipboard still holds what we put there).
#[tauri::command]
pub fn copy(app: AppHandle, state: State<AppState>, id: String, which: String) -> Result<(), String> {
    let value = field(&state, &id, &which)?;
    app.clipboard().write_text(value.clone()).map_err(|e| e.to_string())?;

    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(Duration::from_secs(CLIPBOARD_CLEAR_SECS)).await;
        if app.clipboard().read_text().ok().as_deref() == Some(value.as_str()) {
            let _ = app.clipboard().write_text(String::new());
        }
    });
    Ok(())
}

/// Download a fresh copy from the server and save it in the vault.
/// If this fails (offline), the old copy stays as it was.
#[tauri::command]
pub async fn sync(app: AppHandle, state: State<'_, AppState>) -> Result<i64, String> {
    // Clone what we need and release the lock before the slow network call.
    let creds = {
        let guard = state.0.lock().unwrap();
        guard.as_ref().ok_or("Vault is locked")?.data.credentials.clone()
    };
    let snapshot = Client::new(creds)?.fetch_all().await?;
    let synced_at = snapshot.synced_at;

    let path = vault_path(&app)?;
    let mut guard = state.0.lock().unwrap();
    let s = guard.as_mut().ok_or("Vault was locked during sync")?;
    s.data.snapshot = Some(snapshot);
    vault::save(&path, &s.key, &s.salt, &s.data)?;
    Ok(synced_at)
}

