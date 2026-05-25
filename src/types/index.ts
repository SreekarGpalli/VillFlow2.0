export interface AppSettings {
  // General
  launch_at_startup: boolean;
  start_minimized: boolean;
  show_pill_overlay: boolean;
  play_sound_on_complete: boolean;
  completion_sound_path: string;
  show_notification_on_error: boolean;
  // Hotkeys
  ptt_stt_hotkey: string;
  ptt_command_hotkey: string;
  open_settings_hotkey: string;
  // Audio
  input_device: string;
  // LLM
  llm_model: string;
  llm_temperature: number;
  llm_max_tokens: number;
  stt_cleanup_prompt: string;
  command_mode_prompt: string;
  // API
  speechmatics_region: 'eu' | 'usa' | 'au';
  speechmatics_operating_point: 'standard' | 'enhanced';
  language: string;
  // Output
  injection_method: 'clipboard' | 'keyboard';
  restore_clipboard: boolean;
  clipboard_restore_delay_ms: number;
  clipboard_restore_attempts: number;
  append_trailing_space: boolean;
}

export interface AudioDevice {
  name: string;
  is_default: boolean;
}

export type PillState = 'recording' | 'processing' | 'success' | 'error';

export interface PillPayload {
  state: PillState;
  message?: string;
}

export type SettingsTab =
  | 'general'
  | 'hotkeys'
  | 'audio'
  | 'apikeys'
  | 'prompts'
  | 'output'
  | 'about';
