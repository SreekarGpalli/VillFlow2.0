//! Application settings stored as TOML in `%APPDATA%\VillFlow\config.toml`.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;

// ---------------------------------------------------------------------------
// Error
// ---------------------------------------------------------------------------

/// Errors that can occur while loading or saving settings.
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("TOML serialization error: {0}")]
    TomlSerialize(#[from] toml::ser::Error),

    #[error("TOML deserialization error: {0}")]
    TomlDeserialize(#[from] toml::de::Error),

    #[error("Could not determine application data directory")]
    NoAppDataDir,
}

// ---------------------------------------------------------------------------
// Paths
// ---------------------------------------------------------------------------

/// Returns `%APPDATA%\VillFlow` (e.g. `C:\Users\<user>\AppData\Roaming\VillFlow`).
pub fn config_dir() -> Result<PathBuf, ConfigError> {
    let base = dirs::config_dir().ok_or(ConfigError::NoAppDataDir)?;
    Ok(base.join("VillFlow"))
}

/// Returns `%APPDATA%\VillFlow\logs`.
pub fn log_dir() -> Result<PathBuf, ConfigError> {
    Ok(config_dir()?.join("logs"))
}

/// Returns the full path to the config file: `%APPDATA%\VillFlow\config.toml`.
fn config_file_path() -> Result<PathBuf, ConfigError> {
    Ok(config_dir()?.join("config.toml"))
}

// ---------------------------------------------------------------------------
// Settings struct
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum InjectionMethod { Clipboard, Keyboard }

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SpeechmaticsRegion { Eu, Usa, Au }

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum OperatingPoint { Standard, Enhanced }

/// All user-configurable settings for VillFlow.
///
/// Every field has a sensible default (see [`Default`] impl).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct AppSettings {
    // ── General ──────────────────────────────────────────────────────────
    /// Start the app automatically when the user logs in.
    pub launch_at_startup: bool,
    /// Start minimized to the system tray.
    pub start_minimized: bool,
    /// Show the floating pill overlay during recording / processing.
    pub show_pill_overlay: bool,
    /// Play an audible chime when dictation completes.
    pub play_sound_on_complete: bool,
    /// Path to a custom WAV file to play on completion, or empty for default.
    pub completion_sound_path: String,
    /// Display a desktop notification on errors.
    pub show_notification_on_error: bool,

    // ── Hotkeys ──────────────────────────────────────────────────────────
    /// Push-to-talk hotkey for STT (dictation) mode.
    pub ptt_stt_hotkey: String,
    /// Push-to-talk hotkey for command mode.
    pub ptt_command_hotkey: String,
    /// Hotkey to open the settings window.
    pub open_settings_hotkey: String,

    // ── Audio ────────────────────────────────────────────────────────────
    /// Name of the input device to use, or `"default"`.
    pub input_device: String,

    // ── LLM ──────────────────────────────────────────────────────────────
    /// Model identifier sent to the Groq API.
    pub llm_model: String,
    /// Temperature for LLM generation.
    pub llm_temperature: f32,
    /// Maximum tokens for LLM generation.
    pub llm_max_tokens: u32,
    /// System prompt used to clean up raw STT transcripts.
    pub stt_cleanup_prompt: String,
    /// System prompt for command-mode interpretation.
    pub command_mode_prompt: String,

    // ── API / Speechmatics ───────────────────────────────────────────────
    /// Speechmatics region.
    pub speechmatics_region: SpeechmaticsRegion,
    /// Speechmatics operating point.
    pub speechmatics_operating_point: OperatingPoint,
    /// Language code for speech recognition (e.g. `"en"`, `"de"`, `"fr"`).
    pub language: String,

    // ── Output / Injection ───────────────────────────────────────────────
    /// Text injection method.
    pub injection_method: InjectionMethod,
    /// Whether to restore the clipboard contents after clipboard-based injection.
    pub restore_clipboard: bool,
    /// Delay in ms before restoring the clipboard.
    pub clipboard_restore_delay_ms: u64,
    /// Number of attempts when restoring clipboard.
    pub clipboard_restore_attempts: u32,
    /// Append a trailing space after injected text.
    pub append_trailing_space: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            // General
            launch_at_startup: true,
            start_minimized: false,
            show_pill_overlay: true,
            play_sound_on_complete: false,
            completion_sound_path: "".into(),
            show_notification_on_error: true,

            // Hotkeys
            ptt_stt_hotkey: "Ctrl+Shift+Z".into(),
            ptt_command_hotkey: "Ctrl+Shift+X".into(),
            open_settings_hotkey: "Alt+,".into(),

            // Audio
            input_device: "default".into(),

            // LLM
            llm_model: "llama-3.3-70b-versatile".into(),
            llm_temperature: 0.2,
            llm_max_tokens: 2048,
            stt_cleanup_prompt: crate::llm::groq::DEFAULT_STT_CLEANUP_PROMPT.into(),
            command_mode_prompt: crate::llm::groq::DEFAULT_COMMAND_MODE_PROMPT.into(),

            // Speechmatics
            speechmatics_region: SpeechmaticsRegion::Eu,
            speechmatics_operating_point: OperatingPoint::Enhanced,
            language: "en".into(),

            // Output
            injection_method: InjectionMethod::Clipboard,
            restore_clipboard: true,
            clipboard_restore_delay_ms: 500,
            clipboard_restore_attempts: 5,
            append_trailing_space: true,
        }
    }
}

// ---------------------------------------------------------------------------
// Load / save
// ---------------------------------------------------------------------------

/// Load settings from disk. Returns `Default` if the file doesn't exist yet.
pub fn load_settings() -> Result<AppSettings, ConfigError> {
    let path = config_file_path()?;
    if !path.exists() {
        tracing::info!("Config file not found; using defaults");
        return Ok(AppSettings::default());
    }
    let contents = std::fs::read_to_string(&path)?;
    let settings: AppSettings = toml::from_str(&contents)?;
    tracing::info!("Loaded settings from {}", path.display());
    Ok(settings)
}

/// Save settings to disk atomically, creating the directory if necessary.
pub fn save_settings(settings: &AppSettings) -> Result<(), ConfigError> {
    let dir = config_dir()?;
    std::fs::create_dir_all(&dir)?;
    let path = config_file_path()?;
    let temp_path = dir.join("config.toml.tmp");
    let contents = toml::to_string_pretty(settings)?;
    std::fs::write(&temp_path, contents)?;
    std::fs::rename(&temp_path, &path)?;
    tracing::info!("Saved settings atomically to {}", path.display());
    Ok(())
}

/// Helper to register or unregister application in Windows startup registry.
pub fn register_startup(enabled: bool) -> Result<(), String> {
    use winreg::enums::{HKEY_CURRENT_USER, KEY_WRITE};
    use winreg::RegKey;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let path = r"Software\Microsoft\Windows\CurrentVersion\Run";
    
    let exe_path = std::env::current_exe()
        .map_err(|e| format!("Failed to get current exe path: {e}"))?;
        
    let key = hkcu.open_subkey_with_flags(path, KEY_WRITE)
        .map_err(|e| format!("Failed to open registry key: {e}"))?;
        
    if enabled {
        key.set_value("VillFlow", &exe_path.to_string_lossy().as_ref())
            .map_err(|e| format!("Failed to write startup key: {e}"))?;
    } else {
        let _ = key.delete_value("VillFlow");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_round_trips_through_toml() {
        let original = AppSettings::default();
        let serialized = toml::to_string_pretty(&original).unwrap();
        let deserialized: AppSettings = toml::from_str(&serialized).unwrap();
        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_register_startup() {
        // Test enabling startup
        let res = register_startup(true);
        assert!(res.is_ok(), "Failed to enable startup: {:?}", res.err());

        // Verify key exists via winreg
        use winreg::enums::{HKEY_CURRENT_USER, KEY_READ};
        use winreg::RegKey;
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let path = r"Software\Microsoft\Windows\CurrentVersion\Run";
        let key = hkcu.open_subkey_with_flags(path, KEY_READ).unwrap();
        let val: String = key.get_value("VillFlow").unwrap();
        assert!(!val.is_empty());

        // Test disabling startup
        let res = register_startup(false);
        assert!(res.is_ok(), "Failed to disable startup: {:?}", res.err());

        // Verify key no longer exists
        let val_err = key.get_value::<String, _>("VillFlow");
        assert!(val_err.is_err(), "VillFlow startup key should have been deleted");
    }
}
