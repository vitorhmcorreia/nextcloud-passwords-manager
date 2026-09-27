//! Copy secrets to the system clipboard without leaving them in clipboard history.
//!
//! We use `arboard` directly (instead of Tauri's clipboard plugin) because it
//! can flag the data as "don't keep in history":
//! - Linux: adds the `x-kde-passwordManagerHint: secret` MIME type, honoured by
//!   KDE's Klipper and other clipboard managers (Wayland and X11).
//! - Windows: excludes it from clipboard history (Win+V) and cloud sync.
//! - macOS: marks it as concealed/transient for clipboard managers.
//!
//! This matters for clearing too: Klipper's default "prevent empty clipboard"
//! puts the last history item back when the clipboard is emptied. If the secret
//! was in its history, "clearing" would just restore the password.

use std::sync::Mutex;

#[cfg(target_os = "macos")]
use arboard::SetExtApple;
#[cfg(target_os = "linux")]
use arboard::SetExtLinux;
#[cfg(windows)]
use arboard::SetExtWindows;

/// One long-lived clipboard handle. On Linux the process that set the
/// clipboard must keep serving it, so we don't create and drop one per copy.
#[derive(Default)]
pub struct Clipboard(Mutex<Option<arboard::Clipboard>>);

impl Clipboard {
    /// Run `f` with the clipboard handle, creating it on first use.
    fn with<T>(&self, f: impl FnOnce(&mut arboard::Clipboard) -> Result<T, arboard::Error>) -> Result<T, String> {
        let mut guard = self.0.lock().unwrap();
        if guard.is_none() {
            *guard = Some(arboard::Clipboard::new().map_err(|e| e.to_string())?);
        }
        f(guard.as_mut().unwrap()).map_err(|e| e.to_string())
    }

    pub fn set_secret(&self, text: &str) -> Result<(), String> {
        self.with(|c| c.set().exclude_from_history().text(text))
    }

    /// Clear the clipboard, but only if it still holds `text`
    /// (don't wipe something the user copied afterwards).
    pub fn clear_if(&self, text: &str) -> Result<(), String> {
        self.with(|c| {
            if c.get_text().ok().as_deref() == Some(text) {
                c.clear()?;
            }
            Ok(())
        })
    }
}

/// Manual check against the real desktop clipboard (KDE: also asks Klipper).
///   cargo test clipboard -- --ignored --nocapture
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]
    fn secret_is_not_kept_after_clear() {
        let cb = Clipboard::default();
        let secret = format!("ncpass-dummy-{}", std::process::id());
        cb.set_secret(&secret).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(500));
        assert_eq!(cb.with(|c| c.get_text()).unwrap(), secret);

        let klipper = || {
            std::process::Command::new("qdbus6")
                .args(["org.kde.klipper", "/klipper", "getClipboardHistoryItem", "0"])
                .output()
                .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
                .unwrap_or_default()
        };
        println!("klipper newest item is the secret: {}", klipper() == secret);

        cb.clear_if(&secret).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(500));
        let after = cb.with(|c| c.get_text()).unwrap_or_default();
        println!("clipboard after clear still holds secret: {}", after == secret);
        assert_ne!(after, secret);
        assert_ne!(klipper(), secret);
    }
}
