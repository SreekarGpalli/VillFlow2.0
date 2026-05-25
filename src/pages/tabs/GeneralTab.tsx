import type { AppSettings } from '../../types';
import { ToggleSwitch } from '../../components/ToggleSwitch';
import { SectionHeader } from '../../components/SectionHeader';

interface GeneralTabProps {
  settings: AppSettings;
  updateSettings: (patch: Partial<AppSettings>) => void;
}

export function GeneralTab({ settings, updateSettings }: GeneralTabProps) {
  return (
    <section
      id="settings-panel-general"
      role="tabpanel"
      aria-labelledby="settings-tab-general"
      className="animate-slide-up"
    >
      <SectionHeader
        title="General"
        subtitle="Configure startup behavior and notifications"
      />
      <div className="divide-y divide-border/50">
        <ToggleSwitch
          id="toggle-launch-startup"
          label="Launch at startup"
          description="Automatically start VillFlow when you log in"
          checked={settings.launch_at_startup}
          onChange={(v) => updateSettings({ launch_at_startup: v })}
        />
        <ToggleSwitch
          id="toggle-start-minimized"
          label="Start minimized"
          description="Start in the system tray without showing a window"
          checked={settings.start_minimized}
          onChange={(v) => updateSettings({ start_minimized: v })}
        />
        <ToggleSwitch
          id="toggle-show-pill"
          label="Show pill overlay"
          description="Display a floating status indicator during recording"
          checked={settings.show_pill_overlay}
          onChange={(v) => updateSettings({ show_pill_overlay: v })}
        />
        <ToggleSwitch
          id="toggle-sound-complete"
          label="Play sound on complete"
          description="Play an audio cue when transcription finishes"
          checked={settings.play_sound_on_complete}
          onChange={(v) => updateSettings({ play_sound_on_complete: v })}
        />
        {settings.play_sound_on_complete && (
          <div className="pl-6 pr-4 pb-4 animate-fade-in">
            <label className="block text-xs font-semibold text-text-primary mb-1">
              Completion Sound Path (WAV)
            </label>
            <input
              type="text"
              className="w-full px-3 py-2 rounded-lg bg-surface-alt border border-border text-xs text-text-primary focus:outline-none focus:border-primary"
              placeholder="Leave empty for default system sound..."
              value={settings.completion_sound_path || ''}
              onChange={(e) => updateSettings({ completion_sound_path: e.target.value })}
            />
          </div>
        )}
        <ToggleSwitch
          id="toggle-notify-error"
          label="Show notification on error"
          description="Display a system notification when something goes wrong"
          checked={settings.show_notification_on_error}
          onChange={(v) => updateSettings({ show_notification_on_error: v })}
        />
      </div>
    </section>
  );
}
