//! Tauri commands for VillFlow.

use tauri::State;
use std::sync::Arc;
use crate::AppState;
use crate::config::{AppSettings, save_settings as save_settings_to_disk, config_dir};
use crate::credentials::{save_key, load_key};
use crate::audio::{enumerate_devices, AudioDevice};

/// Helper to register or unregister application in Windows startup registry.
fn register_startup(enabled: bool) -> Result<(), String> {
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

#[tauri::command]
pub async fn get_config_path() -> Result<String, String> {
    let path = config_dir().map_err(|e| e.to_string())?.join("config.toml");
    Ok(path.to_string_lossy().into_owned())
}

#[tauri::command]
pub async fn get_settings(state: State<'_, Arc<AppState>>) -> Result<AppSettings, String> {
    let settings = state.settings.read().clone();
    Ok(settings)
}

#[tauri::command]
pub async fn save_settings(state: State<'_, Arc<AppState>>, settings: AppSettings) -> Result<(), String> {
    // Enum validation using pattern matching
    match settings.injection_method {
        crate::config::InjectionMethod::Clipboard => {}
        crate::config::InjectionMethod::Keyboard => {}
    }

    // Save settings to config.toml
    save_settings_to_disk(&settings).map_err(|e| e.to_string())?;

    // Update Windows startup registry if changed
    let old_launch = state.settings.read().launch_at_startup;
    if old_launch != settings.launch_at_startup {
        register_startup(settings.launch_at_startup)?;
    }

    // Check if hotkey settings changed before rebuilding
    let hotkeys_changed = {
        let old = state.settings.read();
        old.ptt_stt_hotkey != settings.ptt_stt_hotkey
            || old.ptt_command_hotkey != settings.ptt_command_hotkey
            || old.open_settings_hotkey != settings.open_settings_hotkey
    };

    // Update settings in memory
    *state.settings.write() = settings;

    // Restart hotkeys only if hotkey settings changed
    if hotkeys_changed {
        let handle = state.app_handle.clone();
        tauri::async_runtime::spawn(async move {
            if let Err(e) = crate::rebuild_hotkeys(&handle).await {
                tracing::error!("Failed to rebuild hotkeys: {e}");
            }
        });
    }

    Ok(())
}

#[tauri::command]
pub async fn get_api_key(service: String) -> Result<Option<String>, String> {
    if service != "speechmatics" && service != "groq" {
        return Err("Unauthorized service credential requested".to_string());
    }
    load_key(&service).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_api_key(service: String, value: String) -> Result<(), String> {
    if service != "speechmatics" && service != "groq" {
        return Err("Unauthorized service credential set requested".to_string());
    }
    if value.len() < 8 || value.len() > 512 {
        return Err("API key must be between 8 and 512 characters".to_string());
    }
    if !value.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.') {
        return Err("API key contains invalid characters (only alphanumeric, hyphens, underscores, and periods allowed)".to_string());
    }
    save_key(&service, &value).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_audio_devices() -> Result<Vec<AudioDevice>, String> {
    enumerate_devices().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn open_log_folder() -> Result<(), String> {
    let log_path = crate::config::log_dir().map_err(|e| e.to_string())?;
    if !log_path.exists() {
        std::fs::create_dir_all(&log_path).map_err(|e| e.to_string())?;
    }
    
    // Spawn Windows Explorer
    std::process::Command::new("explorer")
        .arg(&log_path)
        .spawn()
        .map_err(|e| e.to_string())?;
        
    Ok(())
}

#[tauri::command]
pub async fn reset_all_settings(state: State<'_, Arc<AppState>>) -> Result<(), String> {
    let default_settings = AppSettings::default();
    
    save_settings_to_disk(&default_settings).map_err(|e| e.to_string())?;

    let old_launch = state.settings.read().launch_at_startup;
    if old_launch != default_settings.launch_at_startup {
        register_startup(default_settings.launch_at_startup)?;
    }

    *state.settings.write() = default_settings;

    // Restart hotkeys with default settings
    let handle = state.app_handle.clone();
    tauri::async_runtime::spawn(async move {
        if let Err(e) = crate::rebuild_hotkeys(&handle).await {
            tracing::error!("Failed to rebuild hotkeys: {e}");
        }
    });

    Ok(())
}

#[tauri::command]
pub async fn list_groq_models(state: State<'_, Arc<AppState>>) -> Result<Vec<String>, String> {
    let groq_key = load_key("groq")
        .map_err(|e| e.to_string())?
        .unwrap_or_default();

    if groq_key.trim().is_empty() {
        return Ok(get_default_groq_models().await);
    }

    match crate::llm::groq::fetch_models(&state.http_client, &groq_key).await {
        Ok(models) => Ok(models),
        Err(e) => {
            tracing::warn!("Failed to fetch models from Groq: {e}. Using fallback list.");
            Ok(get_default_groq_models().await)
        }
    }
}

#[derive(serde::Serialize)]
pub struct DefaultPrompts {
    pub stt_cleanup_prompt: String,
    pub command_mode_prompt: String,
}

#[tauri::command]
pub async fn get_default_prompts() -> DefaultPrompts {
    DefaultPrompts {
        stt_cleanup_prompt: crate::llm::groq::DEFAULT_STT_CLEANUP_PROMPT.to_string(),
        command_mode_prompt: crate::llm::groq::DEFAULT_COMMAND_MODE_PROMPT.to_string(),
    }
}

#[tauri::command]
pub async fn get_default_settings() -> AppSettings {
    AppSettings::default()
}

#[tauri::command]
pub async fn get_default_groq_models() -> Vec<String> {
    vec![
        "llama-3.3-70b-versatile".to_string(),
        "llama-3.1-8b-instant".to_string(),
        "llama-3.1-70b-versatile".to_string(),
        "mixtral-8x7b-32768".to_string(),
        "gemma2-9b-it".to_string(),
    ]
}
