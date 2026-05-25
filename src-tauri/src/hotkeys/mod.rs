//! Global hotkey management module.

mod manager;

pub use manager::{
    reset_recording_state, start_hotkey_listener, HotkeyError, HotkeyEvent,
    HotkeyManagerHandle, HotkeySettings, PttMode,
};
