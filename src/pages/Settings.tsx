import { useState, useCallback, useEffect, useRef, useMemo } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { getVersion } from '@tauri-apps/api/app';
import { useSettings } from '../hooks/useSettings';
import { ConfirmDialog } from '../components/ConfirmDialog';
import { Logo } from '../components/Logo';
import type { SettingsTab, AudioDevice } from '../types';

import { GeneralTab } from './tabs/GeneralTab';
import { HotkeysTab } from './tabs/HotkeysTab';
import { AudioTab } from './tabs/AudioTab';
import { ApiKeysTab } from './tabs/ApiKeysTab';
import { PromptsTab } from './tabs/PromptsTab';
import { OutputTab } from './tabs/OutputTab';
import { AboutTab } from './tabs/AboutTab';

/* ------------------------------------------------------------------ */
/*  Tab definitions                                                    */
/* ------------------------------------------------------------------ */

interface TabDef {
  id: SettingsTab;
  label: string;
  icon: React.ReactNode;
}

const TABS: TabDef[] = [
  {
    id: 'general',
    label: 'General',
    icon: (
      <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.066 2.573c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.573 1.066c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.066-2.573c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
      </svg>
    ),
  },
  {
    id: 'hotkeys',
    label: 'Hotkeys',
    icon: (
      <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M6.75 7.5l3 2.25-3 2.25m4.5 0h3m-9 8.25h13.5A2.25 2.25 0 0021 18V6a2.25 2.25 0 00-2.25-2.25H5.25A2.25 2.25 0 003 6v12a2.25 2.25 0 002.25 2.25z" />
      </svg>
    ),
  },
  {
    id: 'audio',
    label: 'Audio Input',
    icon: (
      <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M12 18.75a6 6 0 006-6v-1.5m-6 7.5a6 6 0 01-6-6v-1.5m6 7.5v3.75m-3.75 0h7.5M12 15.75a3 3 0 01-3-3V4.5a3 3 0 116 0v8.25a3 3 0 01-3 3z" />
      </svg>
    ),
  },
  {
    id: 'apikeys',
    label: 'Services & APIs',
    icon: (
      <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M15.75 5.25a3 3 0 013 3m3 0a6 6 0 01-7.029 5.912c-.563-.097-1.159.026-1.563.43L10.5 17.25H8.25v2.25H6v2.25H2.25v-2.818c0-.597.237-1.17.659-1.591l6.499-6.499c.404-.404.527-1 .43-1.563A6 6 0 1121.75 8.25z" />
      </svg>
    ),
  },
  {
    id: 'prompts',
    label: 'Prompts',
    icon: (
      <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M9.813 15.904L9 18.75l-.813-2.846a4.5 4.5 0 00-3.09-3.09L2.25 12l2.846-.813a4.5 4.5 0 003.09-3.09L9 5.25l.813 2.846a4.5 4.5 0 003.09 3.09L15.75 12l-2.846.813a4.5 4.5 0 00-3.09 3.09zM18.259 8.715L18 9.75l-.259-1.035a3.375 3.375 0 00-2.455-2.456L14.25 6l1.036-.259a3.375 3.375 0 002.455-2.456L18 2.25l.259 1.035a3.375 3.375 0 002.455 2.456L21.75 6l-1.036.259a3.375 3.375 0 00-2.455 2.456zM16.894 20.567L16.5 21.75l-.394-1.183a2.25 2.25 0 00-1.423-1.423L13.5 18.75l1.183-.394a2.25 2.25 0 001.423-1.423l.394-1.183.394 1.183a2.25 2.25 0 001.423 1.423l1.183.394-1.183.394a2.25 2.25 0 00-1.423 1.423z" />
      </svg>
    ),
  },
  {
    id: 'output',
    label: 'Text Output',
    icon: (
      <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M15.666 3.888A2.25 2.25 0 0013.5 2.25h-3c-1.03 0-1.9.693-2.166 1.638m7.332 0c.055.194.084.4.084.612v0a.75.75 0 01-.75.75H9.334a.75.75 0 01-.75-.75v0c0-.212.03-.418.084-.612m7.332 0c.646.049 1.288.11 1.927.184 1.1.128 1.907 1.077 1.907 2.185V19.5a2.25 2.25 0 01-2.25 2.25H6.75A2.25 2.25 0 014.5 19.5V6.257c0-1.108.806-2.057 1.907-2.185a48.208 48.208 0 011.927-.184" />
      </svg>
    ),
  },
  {
    id: 'about',
    label: 'About & System',
    icon: (
      <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M11.25 11.25l.041-.02a.75.75 0 011.063.852l-.708 2.836a.75.75 0 001.063.853l.041-.021M21 12a9 9 0 11-18 0 9 9 0 0118 0zm-9-3.75h.008v.008H12V8.25z" />
      </svg>
    ),
  },
];

/* ------------------------------------------------------------------ */
/*  Settings Page                                                      */
/* ------------------------------------------------------------------ */

export const Settings: React.FC = () => {
  const { settings, loading, error, saveStatus, updateSettings, resetSettings, loadedSuccessfully, defaultPrompts } = useSettings();
  const [activeTab, setActiveTab] = useState<SettingsTab>('general');
  const [audioDevices, setAudioDevices] = useState<AudioDevice[]>([]);
  const [refreshingDevices, setRefreshingDevices] = useState(false);
  const [showResetConfirm, setShowResetConfirm] = useState(false);
  const [configPath, setConfigPath] = useState('');
  const [speechmaticsKey, setSpeechmaticsKey] = useState('');
  const [groqKey, setGroqKey] = useState('');
  const [speechmaticsSaveStatus, setSpeechmaticsSaveStatus] = useState<'idle' | 'saving' | 'saved' | 'error'>('idle');
  const [groqSaveStatus, setGroqSaveStatus] = useState<'idle' | 'saving' | 'saved' | 'error'>('idle');
  const [appVersion, setAppVersion] = useState('2.0.0');
  const [fallbackModels, setFallbackModels] = useState<string[]>([]);
  const [groqModels, setGroqModels] = useState<string[]>([]);

  const speechmaticsKeyTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const groqKeyTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const isMountedRef = useRef(true);

  // Clean up timers on unmount
  useEffect(() => {
    isMountedRef.current = true;
    return () => {
      isMountedRef.current = false;
      if (speechmaticsKeyTimerRef.current) clearTimeout(speechmaticsKeyTimerRef.current);
      if (groqKeyTimerRef.current) clearTimeout(groqKeyTimerRef.current);
    };
  }, []);

  // Load fallback models
  useEffect(() => {
    invoke<string[]>('get_default_groq_models')
      .then((models) => {
        if (isMountedRef.current) setFallbackModels(models);
      })
      .catch(() => {
        if (isMountedRef.current) {
          setFallbackModels([
            'llama-3.3-70b-versatile',
            'llama-3.1-8b-instant',
            'llama-3.1-70b-versatile',
            'mixtral-8x7b-32768',
            'gemma2-9b-it',
          ]);
        }
      });
  }, []);

  // Load API keys
  useEffect(() => {
    invoke<string | null>('get_api_key', { service: 'speechmatics' })
      .then((val) => { if (val && isMountedRef.current) setSpeechmaticsKey(val); })
      .catch((err) => console.warn('Could not load Speechmatics key:', err));

    invoke<string | null>('get_api_key', { service: 'groq' })
      .then((val) => { if (val && isMountedRef.current) setGroqKey(val); })
      .catch((err) => console.warn('Could not load Groq key:', err));
  }, []);

  // Load Groq models with debounce to prevent race conditions and excessive API calls while typing
  useEffect(() => {
    if (!groqKey) {
      if (isMountedRef.current) setGroqModels(fallbackModels);
      return;
    }

    const timer = setTimeout(() => {
      invoke<string[]>('list_groq_models')
        .then((models) => {
          if (isMountedRef.current) setGroqModels(models);
        })
        .catch((err) => {
          console.warn('Could not load Groq models:', err);
          if (isMountedRef.current) setGroqModels(fallbackModels);
        });
    }, 800);

    return () => clearTimeout(timer);
  }, [groqKey, fallbackModels]);

  // Load audio devices
  useEffect(() => {
    invoke<AudioDevice[]>('list_audio_devices')
      .then((devices) => {
        if (isMountedRef.current) setAudioDevices(devices);
      })
      .catch((err) => {
        console.warn('Could not list audio devices:', err);
        if (isMountedRef.current) setAudioDevices([{ name: 'default', is_default: true }]);
      });
  }, []);

  const handleRefreshDevices = useCallback(async () => {
    if (isMountedRef.current) setRefreshingDevices(true);
    try {
      const devices = await invoke<AudioDevice[]>('list_audio_devices');
      if (isMountedRef.current) setAudioDevices(devices);
    } catch (err) {
      console.warn('Could not list audio devices:', err);
    } finally {
      setTimeout(() => {
        if (isMountedRef.current) setRefreshingDevices(false);
      }, 600);
    }
  }, []);

  // Load config path & version
  useEffect(() => {
    invoke<string>('get_config_path')
      .then(setConfigPath)
      .catch(() => setConfigPath('Unknown'));

    getVersion().then(setAppVersion).catch(() => {});
  }, []);

  const handleTabKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      const currentIndex = TABS.findIndex((t) => t.id === activeTab);
      let nextIndex = currentIndex;

      if (e.key === 'ArrowDown' || e.key === 'ArrowRight') {
        e.preventDefault();
        nextIndex = (currentIndex + 1) % TABS.length;
      } else if (e.key === 'ArrowUp' || e.key === 'ArrowLeft') {
        e.preventDefault();
        nextIndex = (currentIndex - 1 + TABS.length) % TABS.length;
      }

      if (nextIndex !== currentIndex) {
        setActiveTab(TABS[nextIndex].id);
        // Focus the new tab button
        const el = document.getElementById(`settings-tab-${TABS[nextIndex].id}`);
        el?.focus();
      }
    },
    [activeTab],
  );

  const handleOpenLogFolder = useCallback(async () => {
    try {
      await invoke('open_log_folder');
    } catch (err) {
      console.error('Failed to open log folder:', err);
    }
  }, []);

  const handleReset = useCallback(async () => {
    await resetSettings();
    setShowResetConfirm(false);
  }, [resetSettings]);

  // Hotkey conflict detection
  const hotkeyConflicts = useMemo(() => {
    if (!settings) return {};
    const hotkeys = [
      { id: 'ptt_stt_hotkey', label: 'Push-to-Talk (STT)', value: settings.ptt_stt_hotkey },
      { id: 'ptt_command_hotkey', label: 'Push-to-Talk (Command)', value: settings.ptt_command_hotkey },
      { id: 'open_settings_hotkey', label: 'Open Settings', value: settings.open_settings_hotkey },
    ];
    const conflicts: Record<string, string> = {};
    for (let i = 0; i < hotkeys.length; i++) {
      for (let j = i + 1; j < hotkeys.length; j++) {
        if (hotkeys[i].value && hotkeys[i].value === hotkeys[j].value) {
          conflicts[hotkeys[i].id] = `Conflicts with ${hotkeys[j].label}`;
          conflicts[hotkeys[j].id] = `Conflicts with ${hotkeys[i].label}`;
        }
      }
    }
    return conflicts;
  }, [settings]);

  const onChangeSpeechmaticsKey = useCallback((val: string) => {
    setSpeechmaticsKey(val);
    if (!loadedSuccessfully) return;
    setSpeechmaticsSaveStatus('saving');
    if (speechmaticsKeyTimerRef.current) clearTimeout(speechmaticsKeyTimerRef.current);
    speechmaticsKeyTimerRef.current = setTimeout(() => {
      invoke('set_api_key', { service: 'speechmatics', value: val })
        .then(() => {
          if (isMountedRef.current) {
            setSpeechmaticsSaveStatus('saved');
            setTimeout(() => {
              if (isMountedRef.current) setSpeechmaticsSaveStatus('idle');
            }, 1500);
          }
        })
        .catch((err) => {
          console.error(err);
          if (isMountedRef.current) setSpeechmaticsSaveStatus('error');
        });
    }, 600);
  }, [loadedSuccessfully]);

  const onChangeGroqKey = useCallback((val: string) => {
    setGroqKey(val);
    if (!loadedSuccessfully) return;
    setGroqSaveStatus('saving');
    if (groqKeyTimerRef.current) clearTimeout(groqKeyTimerRef.current);
    groqKeyTimerRef.current = setTimeout(() => {
      invoke('set_api_key', { service: 'groq', value: val })
        .then(() => {
          if (isMountedRef.current) {
            setGroqSaveStatus('saved');
            setTimeout(() => {
              if (isMountedRef.current) setGroqSaveStatus('idle');
            }, 1500);
          }
        })
        .catch((err) => {
          console.error(err);
          if (isMountedRef.current) setGroqSaveStatus('error');
        });
    }, 600);
  }, [loadedSuccessfully]);

  if (loading || !settings) {
    return (
      <div className="flex items-center justify-center h-screen bg-surface">
        <div className="flex flex-col items-center gap-4">
          <Logo size={56} />
          <div className="w-8 h-8 border-2 border-primary border-t-transparent rounded-full animate-spin" />
          <p className="text-sm text-text-muted">Loading settings…</p>
        </div>
      </div>
    );
  }

  return (
    <div className="flex h-screen bg-surface overflow-hidden select-none">
      {/* ---- Sidebar ---- */}
      <nav
        className="
          w-52 shrink-0 flex flex-col
          bg-sidebar border-r border-border
          pt-6 pb-4
        "
        aria-label="Settings navigation"
      >
        {/* Brand */}
        <div className="px-5 mb-6">
          <div className="flex items-center gap-2.5">
            <Logo size={32} />
            <div>
              <h1 className="text-lg font-bold text-text-primary tracking-tight leading-tight">
                VillFlow
              </h1>
              <p className="text-xs text-text-muted">Settings</p>
            </div>
          </div>
        </div>

        {/* Tab list */}
        <div
          role="tablist"
          aria-label="Settings sections"
          className="flex-1 flex flex-col gap-0.5 px-3"
          onKeyDown={handleTabKeyDown}
        >
          {TABS.map((tab) => {
            const isActive = activeTab === tab.id;
            return (
              <button
                key={tab.id}
                id={`settings-tab-${tab.id}`}
                type="button"
                role="tab"
                aria-selected={isActive}
                aria-controls={`settings-panel-${tab.id}`}
                tabIndex={isActive ? 0 : -1}
                onClick={() => setActiveTab(tab.id)}
                className={`
                  relative flex items-center gap-3 w-full px-3 py-2.5 rounded-lg
                  text-sm font-medium transition-all duration-150 text-left
                  focus-visible:outline-2 focus-visible:outline-offset-2
                  focus-visible:outline-primary
                  ${
                    isActive
                      ? 'bg-primary/10 text-primary font-semibold'
                      : 'text-text-secondary hover:bg-surface-hover hover:text-text-primary'
                  }
                `}
              >
                {isActive && (
                  <span className="absolute left-0 top-2.5 bottom-2.5 w-1 rounded-r bg-primary animate-fade-in" />
                )}
                {tab.icon}
                {tab.label}
              </button>
            );
          })}
        </div>

        {/* Version & Save Status */}
        <div className="px-5 mt-auto pt-4 border-t border-border flex items-center justify-between">
          <p className="text-xs text-text-muted">v{appVersion}</p>
          {saveStatus !== 'idle' && (
            <div className={`flex items-center gap-1.5 text-[10px] font-semibold tracking-wider uppercase animate-fade-in ${
              saveStatus === 'saving' ? 'text-text-muted' :
              saveStatus === 'saved' ? 'text-success' :
              'text-danger'
            }`}>
              {saveStatus === 'saving' && (
                <div className="w-2.5 h-2.5 border-2 border-current border-t-transparent rounded-full animate-spin" />
              )}
              {saveStatus === 'saved' && (
                <svg className="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={3} d="M5 13l4 4L19 7" /></svg>
              )}
              {saveStatus === 'error' && (
                <svg className="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={3} d="M6 18L18 6M6 6l12 12" /></svg>
              )}
              <span>{saveStatus === 'saving' ? 'Saving' : saveStatus === 'saved' ? 'Saved' : 'Error'}</span>
            </div>
          )}
        </div>
      </nav>

      {/* ---- Content ---- */}
      <main className="flex-1 overflow-y-auto">
        <div className="max-w-2xl mx-auto px-8 py-8">
          {error && (
            <div className="mb-6 p-4 rounded-xl bg-danger/10 border border-danger/30 text-danger text-sm flex items-center gap-3 animate-fade-in">
              <svg className="w-5 h-5 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
              </svg>
              <div>
                <p className="font-semibold">Failed to load settings from disk</p>
                <p className="text-xs opacity-90 mt-0.5">Editing is disabled to prevent overwriting your existing settings. {error}</p>
              </div>
            </div>
          )}
          {/* Tab panels */}

          {/* ======== General ======== */}
          {activeTab === 'general' && (
            <GeneralTab settings={settings} updateSettings={updateSettings} />
          )}

          {/* ======== Hotkeys ======== */}
          {activeTab === 'hotkeys' && (
            <HotkeysTab
              settings={settings}
              updateSettings={updateSettings}
              hotkeyConflicts={hotkeyConflicts}
            />
          )}

          {/* ======== Audio ======== */}
          {activeTab === 'audio' && (
            <AudioTab
              settings={settings}
              updateSettings={updateSettings}
              audioDevices={audioDevices}
              refreshingDevices={refreshingDevices}
              handleRefreshDevices={handleRefreshDevices}
            />
          )}

          {/* ======== Prompts ======== */}
          {activeTab === 'prompts' && (
            <PromptsTab
              settings={settings}
              updateSettings={updateSettings}
              defaultPrompts={defaultPrompts}
            />
          )}

          {/* ======== Services & APIs ======== */}
          {activeTab === 'apikeys' && (
            <ApiKeysTab
              settings={settings}
              updateSettings={updateSettings}
              speechmaticsKey={speechmaticsKey}
              speechmaticsSaveStatus={speechmaticsSaveStatus}
              onChangeSpeechmaticsKey={onChangeSpeechmaticsKey}
              groqKey={groqKey}
              groqSaveStatus={groqSaveStatus}
              onChangeGroqKey={onChangeGroqKey}
              groqModels={groqModels}
              fallbackModels={fallbackModels}
            />
          )}

          {/* ======== Output ======== */}
          {activeTab === 'output' && (
            <OutputTab settings={settings} updateSettings={updateSettings} />
          )}

          {/* ======== About ======== */}
          {activeTab === 'about' && (
            <AboutTab
              appVersion={appVersion}
              configPath={configPath}
              handleOpenLogFolder={handleOpenLogFolder}
              setShowResetConfirm={setShowResetConfirm}
            />
          )}
        </div>
      </main>

      {/* Reset confirmation */}
      <ConfirmDialog
        open={showResetConfirm}
        title="Reset All Settings"
        message="This will restore all settings to their default values. Your API keys will not be affected. This action cannot be undone."
        confirmLabel="Reset Settings"
        danger
        onConfirm={handleReset}
        onCancel={() => setShowResetConfirm(false)}
      />
    </div>
  );
}
