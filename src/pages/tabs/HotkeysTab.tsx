import type { AppSettings } from '../../types';
import { KeyCapture } from '../../components/KeyCapture';
import { SectionHeader } from '../../components/SectionHeader';

interface HotkeysTabProps {
  settings: AppSettings;
  updateSettings: (patch: Partial<AppSettings>) => void;
  hotkeyConflicts: Record<string, string>;
}

export function HotkeysTab({ settings, updateSettings, hotkeyConflicts }: HotkeysTabProps) {
  return (
    <section
      id="settings-panel-hotkeys"
      role="tabpanel"
      aria-labelledby="settings-tab-hotkeys"
      className="animate-slide-up"
    >
      <SectionHeader
        title="Hotkeys"
        subtitle="Configure keyboard shortcuts for voice actions"
      />
      <div className="space-y-1">
        <KeyCapture
          id="hotkey-ptt-stt"
          label="Push-to-Talk (Speech-to-Text)"
          value={settings.ptt_stt_hotkey}
          onChange={(v) => updateSettings({ ptt_stt_hotkey: v })}
        />
        {hotkeyConflicts.ptt_stt_hotkey && (
          <div className="flex items-center gap-1.5 text-xs text-warning bg-warning/5 border border-warning/20 rounded-lg p-2.5 -mt-1 mb-2 animate-fade-in">
            <svg className="w-3.5 h-3.5 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 9v2m0 4h.01" />
            </svg>
            <span>{hotkeyConflicts.ptt_stt_hotkey}</span>
          </div>
        )}
        <KeyCapture
          id="hotkey-ptt-command"
          label="Push-to-Talk (Command Mode)"
          value={settings.ptt_command_hotkey}
          onChange={(v) => updateSettings({ ptt_command_hotkey: v })}
        />
        {hotkeyConflicts.ptt_command_hotkey && (
          <div className="flex items-center gap-1.5 text-xs text-warning bg-warning/5 border border-warning/20 rounded-lg p-2.5 -mt-1 mb-2 animate-fade-in">
            <svg className="w-3.5 h-3.5 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 9v2m0 4h.01" />
            </svg>
            <span>{hotkeyConflicts.ptt_command_hotkey}</span>
          </div>
        )}
        <KeyCapture
          id="hotkey-open-settings"
          label="Open Settings"
          value={settings.open_settings_hotkey}
          onChange={(v) => updateSettings({ open_settings_hotkey: v })}
        />
        {hotkeyConflicts.open_settings_hotkey && (
          <div className="flex items-center gap-1.5 text-xs text-warning bg-warning/5 border border-warning/20 rounded-lg p-2.5 -mt-1 mb-2 animate-fade-in">
            <svg className="w-3.5 h-3.5 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 9v2m0 4h.01" />
            </svg>
            <span>{hotkeyConflicts.open_settings_hotkey}</span>
          </div>
        )}
      </div>
    </section>
  );
}
