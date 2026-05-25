import { useState, useEffect, useCallback, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type { AppSettings } from '../types';

const DEBOUNCE_MS: number = 400;
const STATUS_RESET_MS: number = 2000;


interface UseSettingsReturn {
  settings: AppSettings | null;
  loading: boolean;
  error: string | null;
  saveStatus: 'idle' | 'saving' | 'saved' | 'error';
  updateSettings: (patch: Partial<AppSettings>) => void;
  resetSettings: () => Promise<void>;
  loadedSuccessfully: boolean;
  defaultPrompts: { stt_cleanup_prompt: string; command_mode_prompt: string } | null;
}

export function useSettings(): UseSettingsReturn {
  const [settings, setSettings] = useState<AppSettings | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [saveStatus, setSaveStatus] = useState<'idle' | 'saving' | 'saved' | 'error'>('idle');
  const [loadedSuccessfully, setLoadedSuccessfully] = useState(false);
  const [defaultPrompts, setDefaultPrompts] = useState<{ stt_cleanup_prompt: string; command_mode_prompt: string } | null>(null);
  
  const saveTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const saveStatusTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const latestSettingsRef = useRef<AppSettings | null>(null);
  const loadedSuccessfullyRef = useRef(false);

  // Keep ref in sync
  useEffect(() => {
    latestSettingsRef.current = settings;
  }, [settings]);

  // Load settings on mount
  useEffect(() => {
    let cancelled = false;

    async function load() {
      let defaults: { stt_cleanup_prompt: string; command_mode_prompt: string } | null = null;
      try {
        defaults = await invoke<{ stt_cleanup_prompt: string; command_mode_prompt: string }>('get_default_prompts');
        if (!cancelled) {
          setDefaultPrompts(defaults);
        }

        const loaded = await invoke<AppSettings>('get_settings');
        if (!cancelled) {
          setSettings(loaded);
          setLoadedSuccessfully(true);
          loadedSuccessfullyRef.current = true;
          setError(null);
        }
      } catch (err) {
        console.error('Failed to load settings:', err);
        if (!cancelled) {
          // Fallback to defaults so the UI still displays, but prevent saving
          try {
            const fallbackSettings = await invoke<AppSettings>('get_default_settings');
            setSettings(fallbackSettings);
          } catch (e) {
            setSettings({
              launch_at_startup: true,
              start_minimized: false,
              show_pill_overlay: true,
              play_sound_on_complete: false,
              completion_sound_path: '',
              show_notification_on_error: true,
              ptt_stt_hotkey: 'Ctrl+Shift+Z',
              ptt_command_hotkey: 'Ctrl+Shift+X',
              open_settings_hotkey: 'Alt+,',
              input_device: 'default',
              llm_model: 'llama-3.3-70b-versatile',
              llm_temperature: 0.2,
              llm_max_tokens: 2048,
              stt_cleanup_prompt: defaults?.stt_cleanup_prompt || '',
              command_mode_prompt: defaults?.command_mode_prompt || '',
              speechmatics_region: 'eu',
              speechmatics_operating_point: 'enhanced',
              language: 'en',
              injection_method: 'clipboard',
              restore_clipboard: true,
              clipboard_restore_delay_ms: 500,
              clipboard_restore_attempts: 5,
              append_trailing_space: true,
            });
          }
          setLoadedSuccessfully(false);
          loadedSuccessfullyRef.current = false;
          setError(String(err));
        }
      } finally {
        if (!cancelled) {
          setLoading(false);
        }
      }
    }

    load();
    return () => {
      cancelled = true;
    };
  }, []);

  // Debounced save
  const debouncedSave = useCallback((updated: AppSettings) => {
    if (!loadedSuccessfullyRef.current) {
      console.warn('Saving settings is disabled because settings failed to load.');
      return;
    }
    if (saveTimerRef.current) {
      clearTimeout(saveTimerRef.current);
    }
    setSaveStatus('saving');
    saveTimerRef.current = setTimeout(async () => {
      try {
        await invoke('save_settings', { settings: updated });
        setError(null);
        setSaveStatus('saved');
        if (saveStatusTimerRef.current) clearTimeout(saveStatusTimerRef.current);
        saveStatusTimerRef.current = setTimeout(() => setSaveStatus('idle'), STATUS_RESET_MS);
      } catch (err) {
        console.error('Failed to save settings:', err);
        setError(String(err));
        setSaveStatus('error');
      }
    }, DEBOUNCE_MS);
  }, []);

  // Cleanup timer on unmount
  useEffect(() => {
    return () => {
      if (saveTimerRef.current) {
        clearTimeout(saveTimerRef.current);
        if (latestSettingsRef.current && loadedSuccessfullyRef.current) {
          invoke('save_settings', { settings: latestSettingsRef.current }).catch((err) => {
            console.error('Failed to flush settings on unmount:', err);
          });
        }
      }
      if (saveStatusTimerRef.current) {
        clearTimeout(saveStatusTimerRef.current);
      }
    };
  }, []);

  const updateSettings = useCallback(
    (patch: Partial<AppSettings>) => {
      if (!loadedSuccessfullyRef.current) return;
      setSettings((prev) => {
        if (!prev) return prev;
        const updated = { ...prev, ...patch };
        debouncedSave(updated);
        return updated;
      });
    },
    [debouncedSave],
  );

  const resetSettings = useCallback(async () => {
    if (!loadedSuccessfullyRef.current) return;
    try {
      await invoke('reset_all_settings');
      const loaded = await invoke<AppSettings>('get_settings');
      setSettings(loaded);
      setError(null);
    } catch (err) {
      console.error('Failed to reset settings:', err);
      setError(String(err));
    }
  }, []);

  return { settings, loading, error, saveStatus, updateSettings, resetSettings, loadedSuccessfully, defaultPrompts };
}
