# AI Coding Agent Prompt — Fix All Code Quality Issues

Fix every issue listed below in `C:\Users\sreek\Desktop\VC\AG_VillFlow2.0`.

## RULES
- Do NOT add comments unless the fix requires explanation.
- Preserve existing code style.
- After all fixes, `cargo build` and `npm run build` must succeed.

---

## 🔴 HIGH PRIORITY

### 1. Region mismatch bug — `src/pages/Settings.tsx:775` vs `src-tauri/src/stt/speechmatics.rs:143`
The frontend dropdown at `Settings.tsx:775` has `{ value: 'us', label: 'United States (usa)' }` but `speechmatics.rs:143` validates only `"eu" | "usa" | "au"`. The value `"us"` will be rejected at runtime. Fix the frontend to send `"usa"` instead of `"us"`.

### 2. Split monolithic `Settings` component — `src/pages/Settings.tsx:253-1132` (879 lines)
Extract each tab panel into its own component file under `src/pages/tabs/`:
- `src/pages/tabs/GeneralTab.tsx`
- `src/pages/tabs/HotkeysTab.tsx`
- `src/pages/tabs/AudioTab.tsx`
- `src/pages/tabs/ApiKeysTab.tsx`
- `src/pages/tabs/PromptsTab.tsx`
- `src/pages/tabs/OutputTab.tsx`
- `src/pages/tabs/AboutTab.tsx`

Also extract `SectionHeader` into `src/components/SectionHeader.tsx` and `MicrophoneActivityTest` into `src/components/MicrophoneActivityTest.tsx` (currently line 101-247 in Settings.tsx).

### 3. Delete dead code — `src/hooks/useTauriEvent.ts`
The `useTauriEvent` hook is never imported anywhere. Delete the file.

### 4. Decompose `session_task_impl` — `src-tauri/src/stt/speechmatics.rs:146-371` (225 lines)
Split into smaller functions:
- `fn build_ws_request(config: &SpeechmaticsConfig) -> Result<Request, SttError>` (lines 162-183)
- `async fn connect_with_retry(request: Request) -> Result<WebSocket, SttError>` (lines 186-209)
- `async fn read_start_recognition_response(ws_rx: &mut ...) -> Result<(), SttError>` (lines 238-259)
- `async fn stream_audio(ws_tx, audio_rx) -> u64` (lines 262-291)

### 5. Replace stringly-typed enums with real enums
**Rust side:**
- In `src-tauri/src/config/settings.rs`, create Rust enums:
  ```rust
  #[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
  #[serde(rename_all = "lowercase")]
  pub enum InjectionMethod { Clipboard, Keyboard }

  #[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
  #[serde(rename_all = "lowercase")]
  pub enum SpeechmaticsRegion { Eu, Usa, Au }

  #[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
  #[serde(rename_all = "lowercase")]
  pub enum OperatingPoint { Standard, Enhanced }
  ```
- Update `AppSettings` fields to use these enums.
- Update `injection/text_injector.rs` to accept `InjectionMethod` enum instead of `&str`.
- Update `commands/mod.rs` validation to use enum matching instead of string comparison.
- Update `speechmatics.rs` region validation to use the enum.

**TypeScript side:**
- In `src/types/index.ts`, update `AppSettings`:
  ```typescript
  injection_method: 'clipboard' | 'keyboard';
  speechmatics_region: 'eu' | 'usa' | 'au';
  speechmatics_operating_point: 'standard' | 'enhanced';
  ```
- Update `DropdownSelect` options and all onChange handlers.

---

## 🟡 MEDIUM PRIORITY

### 6. Fix `u16` vs `u32` truncation — `src-tauri/src/hotkeys/manager.rs:66`
Change `Hotkey.vk` from `u16` to `u32` to match `windows_sys::Win32::UI::Input::KeyboardAndMouse` VK constant types. Update all comparisons (`vk == VK_CONTROL as u16` → `vk == VK_CONTROL`) and `parse_key_str` return type.

### 7. Move inline imports to module top-level
Move these imports from function bodies to the module-level imports:
- `src-tauri/src/lib.rs:428`: `use futures_util::FutureExt;` → top of file
- `src-tauri/src/stt/speechmatics.rs:150`: `use tokio_tungstenite::tungstenite::Message;` → top of file
- `src-tauri/src/injection/text_injector.rs:182`: `use enigo::Keyboard;` → top of file

### 8. Eliminate duplicate fallback values
- **`src/pages/Settings.tsx:12-18`**: Remove `FALLBACK_GROQ_MODELS` local constant. Load models from backend via `get_default_groq_models` command, or fallback inline.
- **`src/hooks/useSettings.ts:57-81`**: Replace the hardcoded default settings object with a single `invoke('get_default_settings')` call. If that fails, use a minimal fallback. This eliminates the maintenance hazard of duplicating Rust's `AppSettings::default()`.

### 9. Decompose `handle_ptt_up` — `src-tauri/src/lib.rs:298-410` (112 lines)
Extract into named helper functions:
- `async fn wait_for_transcript(timeout: Duration, rx: TranscriptReceiver) -> Result<String, String>`
- `async fn cleanup_with_llm(...) -> Result<String, String>`
- `fn inject_final_text(...) -> Result<(), String>` (lines 386-399)

### 10. Use `TARGET_SAMPLE_RATE` constant — `src-tauri/src/lib.rs:244`
Replace `sample_rate: 16_000` with reference to `crate::audio::capture::TARGET_SAMPLE_RATE` (or make it public in `audio::mod.rs`).

### 11. Name all magic numbers as constants
Add named constants at the top of relevant modules:
- `src-tauri/src/injection/text_injector.rs`: `const CLIPBOARD_RETRIES: u32 = 5;`, `const CLIPBOARD_SETTLE_MS: u64 = 30;`
- `src-tauri/src/stt/speechmatics.rs`: `const RETRY_BASE_DELAY: Duration = Duration::from_secs(2);`, `const RETRY_MAX_DELAY: Duration = Duration::from_secs(8);`, `const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);`, `const READ_TIMEOUT: Duration = Duration::from_secs(30);`
- `src-tauri/src/llm/groq.rs`: `const API_TIMEOUT: Duration = Duration::from_secs(7);`
- `src-tauri/src/overlay/pill.rs`: `const PILL_WIDTH: f64 = 260.0;`, `const PILL_HEIGHT: f64 = 56.0;`, `const AUTO_DISMISS_MS: u64 = 1500;`
- `src/hooks/useSettings.ts`: `const DEBOUNCE_MS: number = 400;`, `const STATUS_RESET_MS: number = 2000;`
- `src/overlay.tsx`: `const FADE_TIMEOUT_MS: number = 1200;`

### 12. Move `PILL_SEQUENCE` and imports to top of file — `src-tauri/src/overlay/pill.rs:114-116`
Move `use std::sync::atomic::{AtomicU32, Ordering}` and `static PILL_SEQUENCE: AtomicU32 = AtomicU32::new(0);` to the top of the file, before any function definitions.

---

## 🔵 LOW PRIORITY

### 13. Remove double blank lines — `src-tauri/src/commands/mod.rs:106-107`, `src-tauri/src/llm/groq.rs:8-10`
Normalize to single blank lines between functions.

### 14. Use shared type for PillPayload — `src/overlay.tsx:13-14`
Change the inline `{ state: PillState; message?: string }` event payload type to import `PillState` from `src/types/index.ts` and properly type the listener with a named interface.

### 15. Replace dynamic import with static import — `src/pages/Settings.tsx:347`
Move `import { getVersion } from '@tauri-apps/api/app'` to the top of the file. Replace the dynamic import with a direct call.

### 16. Replace `.last().unwrap()` with safer pattern — `src-tauri/src/hotkeys/manager.rs:487`
Use `.last().ok_or_else(|| HotkeyError::Parse("Empty hotkey string".into()))?` instead of `.unwrap()`, with the `parts.is_empty()` check already at line 478, but make the last access consistent.
