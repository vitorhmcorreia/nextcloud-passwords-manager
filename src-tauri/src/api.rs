//! Talks to the Nextcloud Passwords REST API (read-only for now).
//!
//! API docs: https://git.mdns.eu/nextcloud/passwords/-/wikis/Developers/Api/Password-Api

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Where the server is and who we log in as. Stored inside the encrypted vault.
#[derive(Clone, Serialize, Deserialize)]
pub struct Credentials {
    pub server: String,
    pub user: String,
    pub password: String,
}

/// One password entry as we keep it locally.
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Password {
    pub id: String,
    pub label: String,
    pub username: String,
    pub password: String,
    pub url: String,
    pub notes: String,
    pub folder: String,
    pub tags: Vec<String>,
    pub favorite: bool,
    pub edited: i64,
    /// JSON-encoded list of custom fields, kept as the server sends it.
    pub custom_fields: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Folder {
    pub id: String,
    pub label: String,
    pub parent: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Tag {
    pub id: String,
    pub label: String,
    pub color: String,
}

/// Everything fetched in one sync.
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub passwords: Vec<Password>,
    pub folders: Vec<Folder>,
    pub tags: Vec<Tag>,
    /// Unix timestamp (seconds) of when the sync happened.
    pub synced_at: i64,
}

// --- Raw shapes returned by the server -------------------------------------
// With `details: "model+tags"` each password carries full tag objects;
// we only keep their ids.

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ApiPassword {
    id: String,
    label: String,
    #[serde(default)]
    username: String,
    #[serde(default)]
    password: String,
    #[serde(default)]
    url: String,
    #[serde(default)]
    notes: String,
    folder: String,
    #[serde(default)]
    tags: Vec<ApiRef>,
    #[serde(default)]
    favorite: bool,
    #[serde(default)]
    trashed: bool,
    #[serde(default)]
    edited: i64,
    #[serde(default)]
    custom_fields: String,
}

#[derive(Deserialize)]
struct ApiRef {
    id: String,
}

#[derive(Deserialize)]
struct ApiFolder {
    id: String,
    label: String,
    parent: String,
    #[serde(default)]
    trashed: bool,
}

#[derive(Deserialize)]
struct ApiTag {
    id: String,
    label: String,
    #[serde(default)]
    color: String,
    #[serde(default)]
    trashed: bool,
}

pub struct Client {
    http: reqwest::Client,
    base: String,
    creds: Credentials,
}

impl Client {
    pub fn new(creds: Credentials) -> Result<Self, String> {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| e.to_string())?;
        let base = format!(
            "{}/index.php/apps/passwords/api/1.0",
            creds.server.trim_end_matches('/')
        );
        Ok(Self { http, base, creds })
    }

    /// POST `{base}/{endpoint}` and parse the JSON reply as `T`.
    async fn list<T: for<'de> Deserialize<'de>>(
        &self,
        endpoint: &str,
        details: &str,
    ) -> Result<T, String> {
        let resp = self
            .http
            .post(format!("{}/{}", self.base, endpoint))
            .basic_auth(&self.creds.user, Some(&self.creds.password))
            .json(&serde_json::json!({ "details": details }))
            .send()
            .await
            .map_err(|e| format!("Could not reach server: {e}"))?;

        match resp.status().as_u16() {
            200 => resp.json::<T>().await.map_err(|e| format!("Unexpected reply: {e}")),
            401 => Err("Login failed: check user and password".into()),
            code => Err(format!("Server replied with HTTP {code}")),
        }
    }

    /// Download all passwords, folders and tags, skipping trashed items.
    pub async fn fetch_all(&self) -> Result<Snapshot, String> {
        // `tokio::try_join!` runs the three requests at the same time and
        // stops at the first error.
        let (pw, fo, ta) = tokio::try_join!(
            self.list::<Vec<ApiPassword>>("password/list", "model+tags"),
            self.list::<Vec<ApiFolder>>("folder/list", "model"),
            self.list::<Vec<ApiTag>>("tag/list", "model"),
        )?;

        let passwords = pw
            .into_iter()
            .filter(|p| !p.trashed)
            .map(|p| Password {
                id: p.id,
                label: p.label,
                username: p.username,
                password: p.password,
                url: p.url,
                notes: p.notes,
                folder: p.folder,
                tags: p.tags.into_iter().map(|t| t.id).collect(),
                favorite: p.favorite,
                edited: p.edited,
                custom_fields: p.custom_fields,
            })
            .collect();
        let folders = fo
            .into_iter()
            .filter(|f| !f.trashed)
            .map(|f| Folder { id: f.id, label: f.label, parent: f.parent })
            .collect();
        let tags = ta
            .into_iter()
            .filter(|t| !t.trashed)
            .map(|t| Tag { id: t.id, label: t.label, color: t.color })
            .collect();

        Ok(Snapshot { passwords, folders, tags, synced_at: now() })
    }
}

pub fn now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}
