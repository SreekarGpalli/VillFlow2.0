import type { AppSettings } from '../../types';
import { ToggleSwitch } from '../../components/ToggleSwitch';
import { DropdownSelect } from '../../components/DropdownSelect';
import { SectionHeader } from '../../components/SectionHeader';

interface OutputTabProps {
  settings: AppSettings;
  updateSettings: (patch: Partial<AppSettings>) => void;
}

export function OutputTab({ settings, updateSettings }: OutputTabProps) {
  return (
    <section
      id="settings-panel-output"
      role="tabpanel"
      aria-labelledby="settings-tab-output"
      className="animate-slide-up"
    >
      <SectionHeader
        title="Text Output"
        subtitle="Configure how transcribed text is delivered"
      />
      <DropdownSelect
        id="select-injection-method"
        label="Injection Method"
        description="How the transcribed text is inserted into the active application"
        value={settings.injection_method}
        options={[
          { value: 'clipboard', label: 'Clipboard (Ctrl+V paste)' },
          { value: 'keyboard', label: 'Keyboard simulation' },
        ]}
        onChange={(v) => updateSettings({ injection_method: v as 'clipboard' | 'keyboard' })}
      />
      
      {settings.injection_method === 'clipboard' ? (
        <div className="animate-fade-in space-y-2">
          <div className="divide-y divide-border/50">
            <ToggleSwitch
              id="toggle-restore-clipboard"
              label="Restore clipboard"
              description="Restore original clipboard contents after pasting"
              checked={settings.restore_clipboard}
              onChange={(v) => updateSettings({ restore_clipboard: v })}
            />
            <ToggleSwitch
              id="toggle-trailing-space"
              label="Append trailing space"
              description="Add a space after the injected text"
              checked={settings.append_trailing_space}
              onChange={(v) => updateSettings({ append_trailing_space: v })}
            />
          </div>
          <DropdownSelect
            id="select-clipboard-delay"
            label="Clipboard Restore Delay"
            description="How long to wait before restoring the original clipboard content. Lower values may cause issues on slower systems."
            value={String(settings.clipboard_restore_delay_ms)}
            options={[
              { value: '50', label: '50 ms' },
              { value: '100', label: '100 ms' },
              { value: '150', label: '150 ms' },
              { value: '200', label: '200 ms' },
              { value: '300', label: '300 ms' },
              { value: '500', label: '500 ms (default)' },
            ]}
            onChange={(v) =>
              updateSettings({ clipboard_restore_delay_ms: parseInt(v, 10) })
            }
          />
          <DropdownSelect
            id="select-clipboard-attempts"
            label="Clipboard Restore Attempts"
            description="Number of retries when attempting to restore the original clipboard content."
            value={String(settings.clipboard_restore_attempts)}
            options={[
              { value: '1', label: '1 retry' },
              { value: '3', label: '3 retries' },
              { value: '5', label: '5 retries (default)' },
              { value: '10', label: '10 retries' },
            ]}
            onChange={(v) =>
              updateSettings({ clipboard_restore_attempts: parseInt(v, 10) })
            }
          />
        </div>
      ) : (
        <div className="animate-fade-in divide-y divide-border/50">
          <ToggleSwitch
            id="toggle-trailing-space"
            label="Append trailing space"
            description="Add a space after the injected text"
            checked={settings.append_trailing_space}
            onChange={(v) => updateSettings({ append_trailing_space: v })}
          />
        </div>
      )}
    </section>
  );
}
