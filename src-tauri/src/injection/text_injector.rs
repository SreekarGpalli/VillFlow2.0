//! Text injection into the currently focused application.
//!
//! Two methods are supported:
//! - **clipboard** (default): copy text to clipboard, simulate Ctrl+V, optionally restore
//! - **keyboard**: type via `enigo` key simulation (fallback)

use thiserror::Error;
use parking_lot::Mutex;
use enigo::{Enigo, Settings, Keyboard};
use crate::config::InjectionMethod;

const CLIPBOARD_RETRIES: u32 = 5;
const CLIPBOARD_SETTLE_MS: u64 = 30;

static ENIGO_INSTANCE: Mutex<Option<Enigo>> = Mutex::new(None);

fn with_enigo<F, R>(f: F) -> Result<R, InjectionError>
where
    F: FnOnce(&mut Enigo) -> Result<R, InjectionError>,
{
    let mut guard = match ENIGO_INSTANCE.try_lock() {
        Some(g) => g,
        None => {
            tracing::warn!("Concurrent text injection detected! Blocking until previous injection completes.");
            ENIGO_INSTANCE.lock()
        }
    };
    if guard.is_none() {
        let enigo = Enigo::new(&Settings::default())
            .map_err(|e| InjectionError::Enigo(e.to_string()))?;
        *guard = Some(enigo);
    }
    f(guard.as_mut().unwrap())
}

// ---------------------------------------------------------------------------
// Error
// ---------------------------------------------------------------------------

/// Errors from text injection.
#[derive(Debug, Error)]
pub enum InjectionError {
    #[error("Clipboard error: {0}")]
    Clipboard(String),

    #[error("Keyboard simulation error: {0}")]
    Keyboard(String),

    #[error("Enigo error: {0}")]
    Enigo(String),
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Inject `text` into the currently focused application.
///
/// # Arguments
/// - `method` — `"clipboard"` or `"keyboard"`
/// - `restore_clipboard` — if true (and method=clipboard), the previous
///   clipboard contents are restored after pasting.
/// - `restore_delay_ms` — how long to wait before restoring the clipboard.
/// - `restore_attempts` — number of retries for clipboard restoration.
/// - `append_space` — if true, a trailing space is appended to the text.
pub fn inject_text(
    text: &str,
    method: InjectionMethod,
    restore_clipboard: bool,
    restore_delay_ms: u64,
    restore_attempts: u32,
    append_space: bool,
) -> Result<(), InjectionError> {
    let mut final_text = text.to_owned();
    if append_space && !final_text.ends_with(' ') {
        final_text.push(' ');
    }

    match method {
        InjectionMethod::Clipboard => inject_via_clipboard(&final_text, restore_clipboard, restore_delay_ms, restore_attempts),
        InjectionMethod::Keyboard => inject_via_keyboard(&final_text),
    }
}

// ---------------------------------------------------------------------------
// Clipboard method
// ---------------------------------------------------------------------------

fn inject_via_clipboard(
    text: &str,
    restore: bool,
    restore_delay_ms: u64,
    restore_attempts: u32,
) -> Result<(), InjectionError> {
    use clipboard_win::{formats, get_clipboard, set_clipboard};

    // Save current clipboard contents if we need to restore, with retries if blocked
    let mut saved: Option<String> = None;
    if restore {
        for _ in 0..CLIPBOARD_RETRIES {
            if let Ok(text) = get_clipboard::<String, _>(formats::Unicode) {
                saved = Some(text);
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }

    // Set new text with retries if clipboard is temporarily locked by other apps
    let mut set_ok = false;
    let mut last_err = String::new();
    for _ in 0..CLIPBOARD_RETRIES {
        match set_clipboard(formats::Unicode, text) {
            Ok(_) => {
                set_ok = true;
                break;
            }
            Err(e) => {
                last_err = e.to_string();
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    if !set_ok {
        return Err(InjectionError::Clipboard(format!("Failed to write to clipboard after retries: {}", last_err)));
    }

    // Small delay to let clipboard settle
    std::thread::sleep(std::time::Duration::from_millis(CLIPBOARD_SETTLE_MS));

    // Simulate Ctrl+V
    simulate_paste()?;

    // Restore clipboard after a delay (in the same blocking scope)
    if restore {
        if let Some(original) = saved {
            std::thread::sleep(std::time::Duration::from_millis(restore_delay_ms));
            let mut restore_ok = false;
            for _ in 0..restore_attempts {
                if set_clipboard(formats::Unicode, &original).is_ok() {
                    restore_ok = true;
                    break;
                }
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
            if restore_ok {
                tracing::debug!("Clipboard restored");
            } else {
                tracing::error!("Failed to restore clipboard after {} retries", restore_attempts);
            }
        }
    }

    tracing::info!("Injected {} chars via clipboard", text.chars().count());
    Ok(())
}

/// Simulate Ctrl+V keypress using `enigo`.
fn simulate_paste() -> Result<(), InjectionError> {
    use enigo::{Direction, Key};

    with_enigo(|enigo| {
        enigo.key(Key::Control, Direction::Press)
            .map_err(|e| InjectionError::Enigo(e.to_string()))?;
        enigo.key(Key::Unicode('v'), Direction::Click)
            .map_err(|e| InjectionError::Enigo(e.to_string()))?;
        enigo.key(Key::Control, Direction::Release)
            .map_err(|e| InjectionError::Enigo(e.to_string()))?;
        Ok(())
    })?;

    // Small delay so the paste has time to take effect
    std::thread::sleep(std::time::Duration::from_millis(50));
    Ok(())
}

// ---------------------------------------------------------------------------
// Keyboard method
// ---------------------------------------------------------------------------

fn inject_via_keyboard(text: &str) -> Result<(), InjectionError> {

    with_enigo(|enigo| {
        enigo.text(text)
            .map_err(|e| InjectionError::Enigo(e.to_string()))?;
        Ok(())
    })?;

    tracing::info!("Injected {} chars via keyboard", text.chars().count());
    Ok(())
}

#[cfg(test)]
mod tests {


    #[test]
    fn test_trailing_space_formatting() {
        let mut text1 = "Hello".to_owned();
        if !text1.ends_with(' ') {
            text1.push(' ');
        }
        assert_eq!(text1, "Hello ");

        let mut text2 = "Hello ".to_owned();
        if !text2.ends_with(' ') {
            text2.push(' ');
        }
        assert_eq!(text2, "Hello ");
    }
}

