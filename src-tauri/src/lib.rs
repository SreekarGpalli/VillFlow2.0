#[cfg(not(target_os = "windows"))]
compile_error!("VillFlow 2.0 is only supported on Windows.");

pub mod audio;
pub mod commands;
pub mod config;
pub mod credentials;
pub mod hotkeys;
pub mod injection;
pub mod llm;
pub mod overlay;
pub mod stt;

use std::sync::Arc;
use parking_lot::{Mutex, RwLock};
use tauri::{AppHandle, Manager};
use hotkeys::{HotkeyEvent, HotkeyManagerHandle, HotkeySettings, PttMode};
use config::AppSettings;
use audio::AudioCaptureHandle;
use stt::TranscriptReceiver;
use futures_util::FutureExt;

/// Active voice session state.
#[derive(Debug)]
pub struct ActiveSession {
    pub audio_handle: AudioCaptureHandle,
    pub transcript_rx: TranscriptReceiver,
    pub mode: PttMode,
    pub start_time: std::time::Instant,
}

#[derive(Debug)]
pub enum SessionState {
    Idle,
    Starting,
    Active(ActiveSession),
}

/// Global Tauri application state.
pub struct AppState {
    pub settings: RwLock<AppSettings>,
    pub app_handle: AppHandle,
    pub hotkey_manager: Mutex<Option<HotkeyManagerHandle>>,
    pub session_state: Mutex<SessionState>,
    pub hotkey_tx: tokio::sync::mpsc::UnboundedSender<HotkeyEvent>,
    pub http_client: reqwest::Client,
    pub _log_guard: Option<tracing_appender::non_blocking::WorkerGuard>,
    pub is_processing: std::sync::atomic::AtomicBool,
}

#[link(name = "winmm")]
extern "system" {
    fn PlaySoundW(
        pszSound: *const u16,
        hmod: isize,
        fdwSound: u32,
    ) -> i32;
}

const SND_FILENAME: u32 = 0x00020000;
const SND_ASYNC: u32 = 0x00000001;
const SND_ALIAS: u32 = 0x00010000;
const SND_NODEFAULT: u32 = 0x00000002;

fn play_completion_sound(sound_path: &str) {
    unsafe {
        if sound_path.trim().is_empty() {
            let sound_name: Vec<u16> = "SystemNotification\0".encode_utf16().collect();
            let res = PlaySoundW(sound_name.as_ptr(), 0, SND_ALIAS | SND_ASYNC | SND_NODEFAULT);
            if res == 0 {
                windows_sys::Win32::System::Diagnostics::Debug::MessageBeep(0x00000040);
            }
        } else {
            let sound_name: Vec<u16> = sound_path.encode_utf16().chain(std::iter::once(0)).collect();
            PlaySoundW(sound_name.as_ptr(), 0, SND_FILENAME | SND_ASYNC);
        }
    }
}

/// Initialize tracing subscriber for VillFlow.
fn init_logging() -> Result<tracing_appender::non_blocking::WorkerGuard, Box<dyn std::error::Error>> {
    let log_path = config::log_dir()?;
    std::fs::create_dir_all(&log_path)?;
    
    let file_appender = tracing_appender::rolling::daily(&log_path, "villflow.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
    
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::DEBUG.into())
        )
        .with_writer(non_blocking)
        .init();
        
    Ok(guard)
}

/// Set up the tray icon with Open Settings and Quit options.
fn setup_tray(app_handle: &tauri::AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    use tauri::menu::{Menu, MenuItem};
    use tauri::tray::{TrayIconBuilder, TrayIconEvent, MouseButton};

    let open_settings = MenuItem::with_id(app_handle, "open_settings", "Open Settings", true, None::<&str>)?;
    let quit = MenuItem::with_id(app_handle, "quit", "Quit", true, None::<&str>)?;
    
    let menu = Menu::with_items(app_handle, &[&open_settings, &quit])?;
    
    let _tray = TrayIconBuilder::with_id("main-tray")
        .menu(&menu)
        .tooltip("VillFlow — Voice Dictation")
        .icon(app_handle.default_window_icon().ok_or("No default window icon")?.clone())
        .on_menu_event(|app, event| {
            match event.id.as_ref() {
                "open_settings" => {
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
                "quit" => {
                    app.exit(0);
                }
                _ => {}
            }
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
        })
        .build(app_handle)?;
        
    Ok(())
}

/// Helper to rebuild hotkey listeners after a settings update.
pub async fn rebuild_hotkeys(app_handle: &tauri::AppHandle) -> Result<(), String> {
    let state = app_handle.state::<Arc<AppState>>().inner().clone();
    tokio::task::spawn_blocking(move || {
        rebuild_hotkeys_internal(&state)
    })
    .await
    .map_err(|e| format!("rebuild thread panicked: {e}"))?
    .map_err(|e| e.to_string())
}

fn rebuild_hotkeys_internal(state: &AppState) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut hkm_guard = state.hotkey_manager.lock();
    if let Some(mut hkm) = hkm_guard.take() {
        hkm.stop();
    }
    
    let settings = state.settings.read();
    let hotkey_settings = HotkeySettings {
        ptt_stt_hotkey: settings.ptt_stt_hotkey.clone(),
        ptt_command_hotkey: settings.ptt_command_hotkey.clone(),
        open_settings_hotkey: settings.open_settings_hotkey.clone(),
    };
    
    let handle = hotkeys::start_hotkey_listener(hotkey_settings, state.hotkey_tx.clone())?;
    *hkm_guard = Some(handle);
    
    Ok(())
}

/// Show desktop notification and visual error overlay when voice flow fails.
fn show_error_feedback(app_handle: &tauri::AppHandle, error_msg: &str) {
    use tauri_plugin_notification::NotificationExt;

    let _ = overlay::show_pill(app_handle, overlay::PillState::Error, Some(error_msg.to_string()));

    let show_notif = {
        let state = app_handle.state::<Arc<AppState>>();
        let val = state.settings.read().show_notification_on_error;
        val
    };

    if show_notif {
        let _ = app_handle.notification()
            .builder()
            .title("VillFlow Error")
            .body(error_msg)
            .show();
    }
}

/// PTT hold: Starts capturing audio and WebSocket STT stream.
async fn handle_ptt_down(state: &Arc<AppState>, mode: PttMode) -> Result<(), String> {
    let res = handle_ptt_down_impl(state, mode).await;
    if res.is_err() {
        hotkeys::reset_recording_state();
    }
    res
}

async fn handle_ptt_down_impl(state: &Arc<AppState>, mode: PttMode) -> Result<(), String> {
    if state.is_processing.load(std::sync::atomic::Ordering::SeqCst) {
        hotkeys::reset_recording_state();
        return Ok(());
    }

    {
        let mut state_guard = state.session_state.lock();
        match *state_guard {
            SessionState::Idle => {
                *state_guard = SessionState::Starting;
            }
            _ => return Ok(()),
        }
    }

    let speechmatics_key_res = credentials::load_key("speechmatics")
        .map_err(|e| e.to_string())
        .and_then(|k| k.ok_or_else(|| "Speechmatics API key not set. Please open settings and add one.".to_string()));

    let mut speechmatics_key = match speechmatics_key_res {
        Ok(key) => key,
        Err(err) => {
            let mut state_guard = state.session_state.lock();
            *state_guard = SessionState::Idle;
            return Err(err);
        }
    };

    let settings = state.settings.read().clone();
    let device_name = if settings.input_device == "default" {
        None
    } else {
        Some(settings.input_device.clone())
    };

    let stt_config = stt::SpeechmaticsConfig {
        api_key: speechmatics_key.clone(),
        region: settings.speechmatics_region,
        operating_point: settings.speechmatics_operating_point,
        language: settings.language.clone(),
        sample_rate: audio::TARGET_SAMPLE_RATE,
    };
    
    let (audio_tx, audio_rx) = tokio::sync::mpsc::channel::<bytes::Bytes>(128);
    let transcript_rx = match stt::start_session(stt_config, audio_rx) {
        Ok(rx) => rx,
        Err(e) => {
            credentials::zeroize_string(&mut speechmatics_key);
            let mut state_guard = state.session_state.lock();
            *state_guard = SessionState::Idle;
            return Err(format!("Failed to start Speechmatics: {e}"));
        }
    };

    let audio_handle = match audio::start_capture(device_name.as_deref(), audio_tx) {
        Ok(handle) => handle,
        Err(e) => {
            credentials::zeroize_string(&mut speechmatics_key);
            let mut state_guard = state.session_state.lock();
            *state_guard = SessionState::Idle;
            return Err(format!("Failed to capture audio: {e}"));
        }
    };

    credentials::zeroize_string(&mut speechmatics_key);

    {
        let mut state_guard = state.session_state.lock();
        *state_guard = SessionState::Active(ActiveSession {
            audio_handle,
            transcript_rx,
            mode,
            start_time: std::time::Instant::now(),
        });
    }

    if settings.show_pill_overlay {
        let _ = overlay::show_pill(&state.app_handle, overlay::PillState::Recording, None);
    }

    Ok(())
}

/// PTT release: Stops capturing audio, awaits transcription, cleans it via LLM, and injects text.
struct ProcessingGuard {
    state: Arc<AppState>,
}

impl Drop for ProcessingGuard {
    fn drop(&mut self) {
        self.state.is_processing.store(false, std::sync::atomic::Ordering::SeqCst);
    }
}

async fn wait_for_transcript(timeout: std::time::Duration, rx: TranscriptReceiver) -> Result<String, String> {
    let transcript_res = tokio::time::timeout(
        timeout,
        rx.wait()
    ).await;

    match transcript_res {
        Ok(Ok(text)) => Ok(text),
        Ok(Err(e)) => Err(format!("Speechmatics session failed: {e}")),
        Err(_) => Err("Speechmatics session timed out".to_string()),
    }
}

async fn cleanup_with_llm(
    state: &Arc<AppState>,
    text: &str,
    mode: PttMode,
    settings: &AppSettings,
) -> Result<String, String> {
    let groq_key = credentials::load_key("groq").unwrap_or(None);
    let mut final_text = text.to_owned();

    if let Some(mut key) = groq_key {
        if !key.is_empty() {
            let default_prompt = match mode {
                PttMode::Stt => llm::groq::DEFAULT_STT_CLEANUP_PROMPT,
                PttMode::Command => llm::groq::DEFAULT_COMMAND_MODE_PROMPT,
            };
            let configured_prompt = match mode {
                PttMode::Stt => &settings.stt_cleanup_prompt,
                PttMode::Command => &settings.command_mode_prompt,
            };
            let system_prompt = if configured_prompt.trim().is_empty() {
                default_prompt
            } else {
                configured_prompt
            };
            
            match llm::cleanup_transcript(
                &state.http_client,
                &final_text,
                system_prompt,
                &key,
                &settings.llm_model,
                settings.llm_temperature,
                settings.llm_max_tokens,
            ).await {
                Ok(cleaned) => {
                    final_text = cleaned;
                }
                Err(e) => {
                    tracing::warn!("Groq LLM cleanup failed: {e}. Using raw text.");
                }
            }
            credentials::zeroize_string(&mut key);
        }
    }
    Ok(final_text)
}

async fn inject_final_text(text: &str, settings: &AppSettings) -> Result<(), String> {
    let settings_clone = settings.clone();
    let final_text_arc = Arc::new(text.to_owned());
    tokio::task::spawn_blocking(move || {
        injection::inject_text(
            &final_text_arc,
            settings_clone.injection_method,
            settings_clone.restore_clipboard,
            settings_clone.clipboard_restore_delay_ms,
            settings_clone.clipboard_restore_attempts,
            settings_clone.append_trailing_space,
        )
    }).await
      .map_err(|e| format!("Injection thread panicked: {e}"))?
      .map_err(|e| format!("Injection failed: {e}"))
}

async fn handle_ptt_up(state: &Arc<AppState>) -> Result<(), String> {
    state.is_processing.store(true, std::sync::atomic::Ordering::SeqCst);
    let _guard = ProcessingGuard { state: state.clone() };

    let session = {
        let mut state_guard = state.session_state.lock();
        match std::mem::replace(&mut *state_guard, SessionState::Idle) {
            SessionState::Active(s) => Some(s),
            _ => None,
        }
    };

    let session = match session {
        Some(s) => s,
        None => return Ok(()),
    };

    let settings = state.settings.read().clone();

    if settings.show_pill_overlay {
        let _ = overlay::update_pill_state(&state.app_handle, overlay::PillState::Processing, None);
    }

    // Stop cpal stream by dropping handle
    drop(session.audio_handle);

    // Wait for websocket transcript with a dynamic timeout
    let recording_duration = session.start_time.elapsed();
    let timeout_secs = (recording_duration.as_secs_f32() * 1.5 + 3.0).max(12.0);
    let timeout_duration = std::time::Duration::from_secs_f32(timeout_secs);

    let raw_transcript = wait_for_transcript(timeout_duration, session.transcript_rx).await?;

    if raw_transcript.is_empty() {
        if settings.show_pill_overlay {
            let _ = overlay::update_pill_state(&state.app_handle, overlay::PillState::Error, Some("No speech detected".to_string()));
        }
        return Err("No speech detected".to_string());
    }

    let final_text = cleanup_with_llm(state, &raw_transcript, session.mode, &settings).await?;

    inject_final_text(&final_text, &settings).await?;

    if settings.show_pill_overlay {
        let _ = overlay::update_pill_state(&state.app_handle, overlay::PillState::Success, None);
    }

    if settings.play_sound_on_complete {
        play_completion_sound(&settings.completion_sound_path);
    }

    Ok(())
}

/// Workers listening to PTT down/up and settings hotkey events.
fn start_hotkey_loop(
    state: Arc<AppState>,
    mut rx: tokio::sync::mpsc::UnboundedReceiver<HotkeyEvent>,
) {
    let app_handle = state.app_handle.clone();
    tauri::async_runtime::spawn(async move {
        while let Some(event) = rx.recv().await {
            match event {
                HotkeyEvent::PttDown { mode, .. } => {
                    let state_clone = state.clone();
                    let app_handle_clone = app_handle.clone();
                    let res = std::panic::AssertUnwindSafe(async move {
                        handle_ptt_down(&state_clone, mode).await
                    });
                    match res.catch_unwind().await {
                        Ok(Ok(())) => {}
                        Ok(Err(e)) => {
                            tracing::error!("PTT Down error: {e}");
                            show_error_feedback(&app_handle_clone, &e);
                        }
                        Err(panic_err) => {
                            tracing::error!("PTT Down panicked: {:?}", panic_err);
                            let mut state_guard = state.session_state.lock();
                            *state_guard = SessionState::Idle;
                            show_error_feedback(&app_handle_clone, "PTT Down handler panicked");
                        }
                    }
                }
                HotkeyEvent::PttUp => {
                    let state_clone = state.clone();
                    let app_handle_clone = app_handle.clone();
                    let res = std::panic::AssertUnwindSafe(async move {
                        handle_ptt_up(&state_clone).await
                    });
                    match res.catch_unwind().await {
                        Ok(Ok(())) => {}
                        Ok(Err(e)) => {
                            tracing::error!("PTT Up error: {e}");
                            show_error_feedback(&app_handle_clone, &e);
                        }
                        Err(panic_err) => {
                            tracing::error!("PTT Up panicked: {:?}", panic_err);
                            state.is_processing.store(false, std::sync::atomic::Ordering::SeqCst);
                            show_error_feedback(&app_handle_clone, "PTT Up handler panicked");
                        }
                    }
                }
                HotkeyEvent::OpenSettings => {
                    if let Some(window) = app_handle.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
            }
        }
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let log_guard = match init_logging() {
        Ok(guard) => Some(guard),
        Err(e) => {
            eprintln!("Failed to initialize logging: {e}. Continuing without file logging.");
            None
        }
    };

    let (hotkey_tx, hotkey_rx) = tokio::sync::mpsc::unbounded_channel::<HotkeyEvent>();

    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .setup(move |app| {
            let app_handle = app.handle().clone();
            
            let settings = config::load_settings().unwrap_or_else(|e| {
                tracing::error!("Failed to load settings: {e}");
                AppSettings::default()
            });

            // Synchronize the startup registry setting on application launch
            if let Err(e) = config::register_startup(settings.launch_at_startup) {
                tracing::error!("Failed to register startup shortcut in registry: {e}");
            }

            let http_client = reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .expect("Failed to build HTTP client");

            let app_state = Arc::new(AppState {
                settings: RwLock::new(settings),
                app_handle: app_handle.clone(),
                hotkey_manager: Mutex::new(None),
                session_state: Mutex::new(SessionState::Idle),
                hotkey_tx: hotkey_tx.clone(),
                http_client,
                _log_guard: log_guard,
                is_processing: std::sync::atomic::AtomicBool::new(false),
            });

            app.manage(app_state.clone());

            let _ = setup_tray(&app_handle);

            start_hotkey_loop(app_state.clone(), hotkey_rx);

            if let Err(e) = rebuild_hotkeys_internal(&app_state) {
                tracing::error!("Failed to register hotkeys on startup: {e}");
            }

            let start_min = app_state.settings.read().start_minimized;
            if !start_min {
                if let Some(window) = app_handle.get_webview_window("main") {
                    let _ = window.show();
                }
            }

            // Force pill overlay WebView background to transparent (fixes rectangular border on Windows)
            if let Some(pill_window) = app_handle.get_webview_window("pill") {
                let _ = pill_window.set_background_color(Some(tauri::webview::Color(0, 0, 0, 0)));
            }

            Ok(())
        })
        .on_window_event(|window, event| {
            if window.label() == "main" {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_config_path,
            commands::get_settings,
            commands::save_settings,
            commands::get_api_key,
            commands::set_api_key,
            commands::list_audio_devices,
            commands::open_log_folder,
            commands::reset_all_settings,
            commands::list_groq_models,
            commands::get_default_prompts,
            commands::get_default_settings,
            commands::get_default_groq_models,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
