//! Global hotkey listener using a raw Win32 keyboard hook (WH_KEYBOARD_LL) on a dedicated OS thread.

use thiserror::Error;
use tokio::sync::mpsc;
use parking_lot::Mutex;
use std::sync::OnceLock;

use windows_sys::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, DispatchMessageW, GetMessageW, SetWindowsHookExW, UnhookWindowsHookEx,
    TranslateMessage, MSG, WH_KEYBOARD_LL, WM_KEYDOWN, WM_KEYUP, WM_SYSKEYDOWN,
    WM_SYSKEYUP, KBDLLHOOKSTRUCT, HHOOK, WM_QUIT, PostThreadMessageW, PeekMessageW, PM_NOREMOVE,
    WM_USER,
};
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    VK_CONTROL, VK_MENU, VK_SHIFT, VK_LWIN, VK_RWIN,
    VK_LCONTROL, VK_RCONTROL, VK_LMENU, VK_RMENU, VK_LSHIFT, VK_RSHIFT,
    VK_CAPITAL, VK_UP, VK_DOWN, VK_LEFT, VK_RIGHT, VK_SPACE, VK_ESCAPE,
    VK_RETURN, VK_TAB, VK_BACK, VK_DELETE, VK_INSERT, VK_HOME, VK_END,
    VK_PRIOR, VK_NEXT, VK_OEM_COMMA, VK_OEM_PERIOD,
    VK_F1, VK_F2, VK_F3, VK_F4, VK_F5, VK_F6, VK_F7, VK_F8, VK_F9, VK_F10, VK_F11, VK_F12,
};

// ---------------------------------------------------------------------------
// Error
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum HotkeyError {
    #[error("Failed to register hotkey: {0}")]
    Registration(String),

    #[error("Failed to parse hotkey string: {0}")]
    Parse(String),

    #[error("Hotkey thread panicked")]
    ThreadPanic,
}

// ---------------------------------------------------------------------------
// Events & Types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PttMode {
    Stt,
    Command,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HotkeyEvent {
    PttDown { mode: PttMode, vk: u32 },
    PttUp,
    OpenSettings,
}

#[derive(Debug, Clone)]
pub struct HotkeySettings {
    pub ptt_stt_hotkey: String,
    pub ptt_command_hotkey: String,
    pub open_settings_hotkey: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Hotkey {
    pub vk: u32,
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub win: bool,
}

// ---------------------------------------------------------------------------
// Shared State & Thread-Locals
// ---------------------------------------------------------------------------

struct HotkeysConfig {
    stt: Option<Hotkey>,
    command: Option<Hotkey>,
    settings: Option<Hotkey>,
}

struct HotkeySharedState {
    hotkeys: HotkeysConfig,
    hook_handle: Option<HHOOK>,
    hook_thread_id: u32,
    event_tx: Option<mpsc::UnboundedSender<HotkeyEvent>>,
    is_recording: bool,
    active_hotkey: Option<Hotkey>,
    swallowed_vk: u32,
}

static SHARED_STATE: OnceLock<Mutex<HotkeySharedState>> = OnceLock::new();

fn get_shared_state() -> &'static Mutex<HotkeySharedState> {
    SHARED_STATE.get_or_init(|| {
        Mutex::new(HotkeySharedState {
            hotkeys: HotkeysConfig {
                stt: None,
                command: None,
                settings: None,
            },
            hook_handle: None,
            hook_thread_id: 0,
            event_tx: None,
            is_recording: false,
            active_hotkey: None,
            swallowed_vk: 0,
        })
    })
}

const MOD_CTRL: u8 = 1 << 0;
const MOD_ALT: u8 = 1 << 1;
const MOD_SHIFT: u8 = 1 << 2;
const MOD_WIN: u8 = 1 << 3;

thread_local! {
    static MODIFIERS: std::cell::Cell<u8> = const { std::cell::Cell::new(0) };
}

// ---------------------------------------------------------------------------
// Handle
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub struct HotkeyManagerHandle {
    thread_handle: Option<std::thread::JoinHandle<()>>,
}

impl HotkeyManagerHandle {
    pub fn stop(&mut self) {
        let hook_thread_id;

        {
            let mut state = get_shared_state().lock();
            state.event_tx = None;
            state.is_recording = false;
            state.active_hotkey = None;
            state.swallowed_vk = 0;

            if let Some(h) = state.hook_handle.take() {
                unsafe {
                    UnhookWindowsHookEx(h);
                }
            }

            hook_thread_id = state.hook_thread_id;
            state.hook_thread_id = 0;
        }

        if hook_thread_id != 0 {
            unsafe {
                PostThreadMessageW(hook_thread_id, WM_QUIT, 0, 0);
            }
        }

        if let Some(h) = self.thread_handle.take() {
            let _ = h.join();
        }
    }
}

impl Drop for HotkeyManagerHandle {
    fn drop(&mut self) {
        self.stop();
    }
}

// ---------------------------------------------------------------------------
// Hook Procedure
// ---------------------------------------------------------------------------

unsafe extern "system" fn low_level_keyboard_hook_proc(
    code: i32,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    let mut h_hook = 0;
    if code >= 0 {
        let msg_type = w_param as u32;
        let kb_struct = *(l_param as *const KBDLLHOOKSTRUCT);
        let vk = kb_struct.vkCode;

        let is_down = msg_type == WM_KEYDOWN || msg_type == WM_SYSKEYDOWN;
        let is_up = msg_type == WM_KEYUP || msg_type == WM_SYSKEYUP;

        if is_down {
            if vk == VK_CONTROL as u32 || vk == VK_LCONTROL as u32 || vk == VK_RCONTROL as u32 {
                MODIFIERS.with(|m| m.set(m.get() | MOD_CTRL));
            } else if vk == VK_MENU as u32 || vk == VK_LMENU as u32 || vk == VK_RMENU as u32 {
                MODIFIERS.with(|m| m.set(m.get() | MOD_ALT));
            } else if vk == VK_SHIFT as u32 || vk == VK_LSHIFT as u32 || vk == VK_RSHIFT as u32 {
                MODIFIERS.with(|m| m.set(m.get() | MOD_SHIFT));
            } else if vk == VK_LWIN as u32 || vk == VK_RWIN as u32 {
                MODIFIERS.with(|m| m.set(m.get() | MOD_WIN));
            }

            let (ctrl, alt, shift, win) = MODIFIERS.with(|m| {
                let val = m.get();
                (
                    (val & MOD_CTRL) != 0,
                    (val & MOD_ALT) != 0,
                    (val & MOD_SHIFT) != 0,
                    (val & MOD_WIN) != 0,
                )
            });

            tracing::trace!("Keyboard Hook Down Event: vk=0x{:X}, ctrl={}, alt={}, shift={}, win={}", vk, ctrl, alt, shift, win);

            let mut state = get_shared_state().lock();
            h_hook = state.hook_handle.unwrap_or(0);
            let hks = &state.hotkeys;
            let mut match_found = None;

            if let Some(ref stt) = hks.stt {
                if vk == stt.vk && ctrl == stt.ctrl && alt == stt.alt && shift == stt.shift && win == stt.win {
                    match_found = Some((PttMode::Stt, stt.clone()));
                }
            }
            if match_found.is_none() {
                if let Some(ref cmd) = hks.command {
                    if vk == cmd.vk && ctrl == cmd.ctrl && alt == cmd.alt && shift == cmd.shift && win == cmd.win {
                        match_found = Some((PttMode::Command, cmd.clone()));
                    }
                }
            }

            if let Some((mode, hotkey)) = match_found {
                let is_rec = state.is_recording;
                if is_rec {
                    tracing::trace!("Hotkey Down ignored: already recording (auto-repeat)");
                    return 1;
                }

                tracing::info!("PTT Down triggered: mode={:?}, vk=0x{:X}", mode, vk);
                state.is_recording = true;
                state.active_hotkey = Some(hotkey);
                state.swallowed_vk = vk;

                if let Some(ref tx) = state.event_tx {
                    let _ = tx.send(HotkeyEvent::PttDown { mode, vk });
                }
                return 1;
            }

            // Check Settings hotkey
            if let Some(ref settings) = hks.settings {
                if vk == settings.vk && ctrl == settings.ctrl && alt == settings.alt && shift == settings.shift && win == settings.win {
                    tracing::info!("Open Settings Hotkey triggered");
                    if let Some(ref tx) = state.event_tx {
                        let _ = tx.send(HotkeyEvent::OpenSettings);
                    }
                    return 1;
                }
            }
        } else if is_up {
            if vk == VK_CONTROL as u32 || vk == VK_LCONTROL as u32 || vk == VK_RCONTROL as u32 {
                MODIFIERS.with(|m| m.set(m.get() & !MOD_CTRL));
            } else if vk == VK_MENU as u32 || vk == VK_LMENU as u32 || vk == VK_RMENU as u32 {
                MODIFIERS.with(|m| m.set(m.get() & !MOD_ALT));
            } else if vk == VK_SHIFT as u32 || vk == VK_LSHIFT as u32 || vk == VK_RSHIFT as u32 {
                MODIFIERS.with(|m| m.set(m.get() & !MOD_SHIFT));
            } else if vk == VK_LWIN as u32 || vk == VK_RWIN as u32 {
                MODIFIERS.with(|m| m.set(m.get() & !MOD_WIN));
            }

            let mut state = get_shared_state().lock();
            h_hook = state.hook_handle.unwrap_or(0);
            let is_rec = state.is_recording;
            tracing::trace!("Keyboard Hook Up Event: vk=0x{:X}, is_recording={}", vk, is_rec);

            let mut should_release = false;

            if is_rec {
                if let Some(ref hotkey) = state.active_hotkey {
                    if vk == hotkey.vk {
                        tracing::info!("PTT Up base key release detected: vk=0x{:X}", vk);
                        should_release = true;
                    }
                    else if hotkey.ctrl && (vk == VK_CONTROL as u32 || vk == VK_LCONTROL as u32 || vk == VK_RCONTROL as u32) {
                        tracing::info!("PTT Up ctrl modifier release detected: vk=0x{:X}", vk);
                        should_release = true;
                    }
                    else if hotkey.alt && (vk == VK_MENU as u32 || vk == VK_LMENU as u32 || vk == VK_RMENU as u32) {
                        tracing::info!("PTT Up alt modifier release detected: vk=0x{:X}", vk);
                        should_release = true;
                    }
                    else if hotkey.shift && (vk == VK_SHIFT as u32 || vk == VK_LSHIFT as u32 || vk == VK_RSHIFT as u32) {
                        tracing::info!("PTT Up shift modifier release detected: vk=0x{:X}", vk);
                        should_release = true;
                    }
                    else if vk == VK_LWIN as u32 || vk == VK_RWIN as u32 {
                        if hotkey.win {
                            tracing::info!("PTT Up win modifier release detected: vk=0x{:X}", vk);
                            should_release = true;
                        }
                    }
                }
            }

            if should_release {
                state.is_recording = false;
                state.active_hotkey = None;

                if let Some(ref tx) = state.event_tx {
                    let _ = tx.send(HotkeyEvent::PttUp);
                }
            }

            let swallowed_vk = state.swallowed_vk;
            if swallowed_vk != 0 && vk == swallowed_vk {
                state.swallowed_vk = 0;
                tracing::info!("Swallowing base key release: vk=0x{:X}", vk);
                return 1;
            }
        }
    }

    if h_hook == 0 {
        h_hook = get_shared_state().lock().hook_handle.unwrap_or(0);
    }
    CallNextHookEx(h_hook, code, w_param, l_param)
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

pub fn reset_recording_state() {
    let mut state = get_shared_state().lock();
    state.is_recording = false;
    state.active_hotkey = None;
}

pub fn start_hotkey_listener(
    settings: HotkeySettings,
    tx: mpsc::UnboundedSender<HotkeyEvent>,
) -> Result<HotkeyManagerHandle, HotkeyError> {
    // Parse hotkeys first to fail fast on errors
    let stt = if !settings.ptt_stt_hotkey.is_empty() {
        Some(parse_hotkey_combo(&settings.ptt_stt_hotkey)?)
    } else {
        None
    };
    let command = if !settings.ptt_command_hotkey.is_empty() {
        Some(parse_hotkey_combo(&settings.ptt_command_hotkey)?)
    } else {
        None
    };
    let settings_hk = if !settings.open_settings_hotkey.is_empty() {
        Some(parse_hotkey_combo(&settings.open_settings_hotkey)?)
    } else {
        None
    };

    {
        let mut state = get_shared_state().lock();
        state.hotkeys = HotkeysConfig {
            stt,
            command,
            settings: settings_hk,
        };
        state.event_tx = Some(tx);
        state.is_recording = false;
        state.active_hotkey = None;
        state.swallowed_vk = 0;
    }

    let (sync_tx, sync_rx) = std::sync::mpsc::channel::<Result<(), String>>();

    let thread_handle = std::thread::Builder::new()
        .name("villflow-hotkeys".into())
        .spawn(move || {
            let thread_id = unsafe { windows_sys::Win32::System::Threading::GetCurrentThreadId() };
            get_shared_state().lock().hook_thread_id = thread_id;

            let h_hook = unsafe {
                let h_instance = windows_sys::Win32::System::LibraryLoader::GetModuleHandleW(std::ptr::null());
                SetWindowsHookExW(
                    WH_KEYBOARD_LL,
                    Some(low_level_keyboard_hook_proc),
                    h_instance,
                    0,
                )
            };

            if h_hook == 0 {
                let _ = sync_tx.send(Err("SetWindowsHookExW failed".into()));
                return;
            }

            unsafe {
                let mut msg: MSG = std::mem::zeroed();
                let _ = PeekMessageW(&mut msg, 0, 0, 0, PM_NOREMOVE);
            }

            get_shared_state().lock().hook_handle = Some(h_hook);

            unsafe {
                PostThreadMessageW(thread_id, WM_USER, 0, 0);
            }

            unsafe {
                let mut msg: MSG = std::mem::zeroed();
                while GetMessageW(&mut msg, 0, 0, 0) > 0 {
                    if msg.message == WM_USER {
                        let _ = sync_tx.send(Ok(()));
                        continue;
                    }
                    TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }
            }
        })
        .map_err(|e| HotkeyError::Registration(e.to_string()))?;

    sync_rx
        .recv()
        .map_err(|_| HotkeyError::ThreadPanic)?
        .map_err(|e| HotkeyError::Registration(e))?;

    Ok(HotkeyManagerHandle {
        thread_handle: Some(thread_handle),
    })
}

// ---------------------------------------------------------------------------
// Key parsing
// ---------------------------------------------------------------------------

fn parse_key_str(key: &str) -> Result<u32, HotkeyError> {
    let key_upper = key.to_uppercase();
    let vk = match key_upper.as_str() {
        "A" => 0x41, "B" => 0x42, "C" => 0x43, "D" => 0x44,
        "E" => 0x45, "F" => 0x46, "G" => 0x47, "H" => 0x48,
        "I" => 0x49, "J" => 0x4A, "K" => 0x4B, "L" => 0x4C,
        "M" => 0x4D, "N" => 0x4E, "O" => 0x4F, "P" => 0x50,
        "Q" => 0x51, "R" => 0x52, "S" => 0x53, "T" => 0x54,
        "U" => 0x55, "V" => 0x56, "W" => 0x57, "X" => 0x58,
        "Y" => 0x59, "Z" => 0x5A,
        "0" => 0x30, "1" => 0x31, "2" => 0x32, "3" => 0x33,
        "4" => 0x34, "5" => 0x35, "6" => 0x36, "7" => 0x37,
        "8" => 0x38, "9" => 0x39,
        "F1" => VK_F1 as u32, "F2" => VK_F2 as u32, "F3" => VK_F3 as u32, "F4" => VK_F4 as u32,
        "F5" => VK_F5 as u32, "F6" => VK_F6 as u32, "F7" => VK_F7 as u32, "F8" => VK_F8 as u32,
        "F9" => VK_F9 as u32, "F10" => VK_F10 as u32, "F11" => VK_F11 as u32, "F12" => VK_F12 as u32,
        "CAPSLOCK" | "CAPS" | "CAPITAL" => VK_CAPITAL as u32,
        "UP" => VK_UP as u32,
        "DOWN" => VK_DOWN as u32,
        "LEFT" => VK_LEFT as u32,
        "RIGHT" => VK_RIGHT as u32,
        "SPACE" => VK_SPACE as u32,
        "ESCAPE" | "ESC" => VK_ESCAPE as u32,
        "ENTER" | "RETURN" => VK_RETURN as u32,
        "TAB" => VK_TAB as u32,
        "BACKSPACE" => VK_BACK as u32,
        "DELETE" | "DEL" => VK_DELETE as u32,
        "INSERT" | "INS" => VK_INSERT as u32,
        "HOME" => VK_HOME as u32,
        "END" => VK_END as u32,
        "PAGEUP" | "PGUP" => VK_PRIOR as u32,
        "PAGEDOWN" | "PGDN" => VK_NEXT as u32,
        "," | "COMMA" => VK_OEM_COMMA as u32,
        "." | "PERIOD" => VK_OEM_PERIOD as u32,
        _ => {
            return Err(HotkeyError::Parse(format!(
                "Unsupported key name: '{}'. Supported keys: A-Z, 0-9, F1-F12, CapsLock, Up, Down, Left, Right, Space, Esc, Enter, Tab, Backspace, Delete, Insert, Home, End, PageUp, PageDown, Comma, Period",
                key
            )));
        }
    };
    Ok(vk)
}

fn parse_hotkey_combo(combo: &str) -> Result<Hotkey, HotkeyError> {
    let parts: Vec<&str> = combo.split('+').map(str::trim).collect();
    if parts.is_empty() {
        return Err(HotkeyError::Parse("Empty hotkey string".into()));
    }

    let mut ctrl = false;
    let mut alt = false;
    let mut shift = false;
    let mut win = false;

    let key_str = parts.last().ok_or_else(|| HotkeyError::Parse("Empty hotkey string".into()))?;
    for &part in &parts[..parts.len() - 1] {
        match part.to_lowercase().as_str() {
            "ctrl" | "control" => ctrl = true,
            "alt" | "menu" => alt = true,
            "shift" => shift = true,
            "win" | "super" | "lwin" => win = true,
            other => return Err(HotkeyError::Parse(format!("Unknown modifier: {other}"))),
        }
    }

    let vk = parse_key_str(key_str)?;

    Ok(Hotkey {
        vk,
        ctrl,
        alt,
        shift,
        win,
    })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hotkey_combo() {
        let hk = parse_hotkey_combo("Ctrl+Z").unwrap();
        assert_eq!(hk.vk, 0x5A);
        assert!(hk.ctrl);
        assert!(!hk.alt);
        assert!(!hk.shift);
        assert!(!hk.win);

        let hk = parse_hotkey_combo("Ctrl+Alt+A").unwrap();
        assert_eq!(hk.vk, 0x41);
        assert!(hk.ctrl);
        assert!(hk.alt);
        assert!(!hk.shift);
        assert!(!hk.win);

        let hk = parse_hotkey_combo("Win+B").unwrap();
        assert_eq!(hk.vk, 0x42);
        assert!(!hk.ctrl);
        assert!(!hk.alt);
        assert!(!hk.shift);
        assert!(hk.win);

        let hk = parse_hotkey_combo("Alt+,").unwrap();
        assert_eq!(hk.vk, VK_OEM_COMMA as u32);
        assert!(!hk.ctrl);
        assert!(hk.alt);
        assert!(!hk.shift);
        assert!(!hk.win);

        let hk = parse_hotkey_combo("CapsLock").unwrap();
        assert_eq!(hk.vk, VK_CAPITAL as u32);
        assert!(!hk.ctrl);
        assert!(!hk.alt);
        assert!(!hk.shift);
        assert!(!hk.win);

        let hk = parse_hotkey_combo("Shift+Up").unwrap();
        assert_eq!(hk.vk, VK_UP as u32);
        assert!(!hk.ctrl);
        assert!(!hk.alt);
        assert!(hk.shift);
        assert!(!hk.win);
    }
}
