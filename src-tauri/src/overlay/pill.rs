//! Overlay pill UI management.
//!
//! The pill is a small, always-on-top overlay window that shows the
//! current state of a dictation operation (recording → processing → done).

use serde::Serialize;
use thiserror::Error;
use std::sync::atomic::{AtomicU32, Ordering};

const PILL_WIDTH: f64 = 260.0;
const PILL_HEIGHT: f64 = 56.0;
const AUTO_DISMISS_MS: u64 = 1500;

static PILL_SEQUENCE: AtomicU32 = AtomicU32::new(0);

// ---------------------------------------------------------------------------
// Error
// ---------------------------------------------------------------------------

/// Errors from pill overlay operations.
#[derive(Debug, Error)]
pub enum PillError {
    #[error("Pill window not found")]
    WindowNotFound,

    #[error("Failed to show pill: {0}")]
    ShowFailed(String),

    #[error("Failed to emit event: {0}")]
    EmitFailed(String),
}

// ---------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------

/// Visual states the pill overlay can display.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum PillState {
    /// Actively recording audio.
    Recording,
    /// Audio captured, processing (STT / LLM).
    Processing,
    /// Operation completed successfully.
    Success,
    /// An error occurred.
    Error,
}

// ---------------------------------------------------------------------------
// Payload sent to the frontend
// ---------------------------------------------------------------------------

/// Payload emitted to the pill window's webview via Tauri events.
#[derive(Debug, Clone, Serialize)]
struct PillPayload {
    state: PillState,
    message: Option<String>,
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Show the pill overlay window and set its initial state.
pub fn show_pill(
    app_handle: &tauri::AppHandle,
    state: PillState,
    message: Option<String>,
) -> Result<(), PillError> {
    use tauri::Manager;

    let window = app_handle
        .get_webview_window("pill")
        .ok_or(PillError::WindowNotFound)?;

    // Position at bottom-center of the primary monitor
    if let Ok(Some(monitor)) = window.primary_monitor() {
        let monitor_size = monitor.size();
        let scale_factor = monitor.scale_factor();
        let logical_width = monitor_size.width as f64 / scale_factor;
        let logical_height = monitor_size.height as f64 / scale_factor;
        let x = (logical_width - PILL_WIDTH) / 2.0;
        let y = logical_height - PILL_HEIGHT - 80.0; // 80px from bottom
        let _ = window.set_position(tauri::Position::Logical(tauri::LogicalPosition::new(x, y)));
    }

    // Update state BEFORE showing the window to avoid a blank flash
    update_pill_state(app_handle, state, message)?;

    window
        .show()
        .map_err(|e| PillError::ShowFailed(e.to_string()))?;
    
    // Make the overlay window click-through so it doesn't intercept clicks
    let _ = window.set_ignore_cursor_events(true);

    tracing::debug!("Pill shown with state: {state:?}");
    Ok(())
}

/// Hide the pill overlay window.
pub fn hide_pill(app_handle: &tauri::AppHandle) -> Result<(), PillError> {
    use tauri::Manager;

    let window = app_handle
        .get_webview_window("pill")
        .ok_or(PillError::WindowNotFound)?;

    window
        .hide()
        .map_err(|e| PillError::ShowFailed(e.to_string()))?;

    tracing::debug!("Pill hidden");
    Ok(())
}



/// Update the pill's visual state by emitting an event.
///
/// For `Success` and `Error` states, an auto-dismiss timer is started (1.5 s).
pub fn update_pill_state(
    app_handle: &tauri::AppHandle,
    state: PillState,
    message: Option<String>,
) -> Result<(), PillError> {
    use tauri::Emitter;

    let seq = PILL_SEQUENCE.fetch_add(1, Ordering::SeqCst) + 1;

    app_handle
        .emit("pill-state", PillPayload { state, message })
        .map_err(|e| PillError::EmitFailed(e.to_string()))?;

    // Auto-dismiss after success or error
    if matches!(state, PillState::Success | PillState::Error) {
        let handle = app_handle.clone();
        tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_millis(AUTO_DISMISS_MS)).await;
            if PILL_SEQUENCE.load(Ordering::SeqCst) == seq {
                if let Err(e) = hide_pill(&handle) {
                    tracing::warn!("Failed to auto-dismiss pill: {e}");
                }
            } else {
                tracing::debug!("Pill sequence changed; skipping auto-dismiss");
            }
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pill_payload_serialization() {
        let payload = PillPayload {
            state: PillState::Recording,
            message: Some("Recording...".to_string()),
        };
        let serialized = serde_json::to_string(&payload).unwrap();
        assert!(serialized.contains(r#""state":"recording""#));
        assert!(serialized.contains(r#""message":"Recording...""#));
    }
}
