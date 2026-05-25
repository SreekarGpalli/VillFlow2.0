import type { AppSettings, AudioDevice } from '../../types';
import { DropdownSelect } from '../../components/DropdownSelect';
import { SectionHeader } from '../../components/SectionHeader';
import { MicrophoneActivityTest } from '../../components/MicrophoneActivityTest';

interface AudioTabProps {
  settings: AppSettings;
  updateSettings: (patch: Partial<AppSettings>) => void;
  audioDevices: AudioDevice[];
  refreshingDevices: boolean;
  handleRefreshDevices: () => Promise<void>;
}

export function AudioTab({
  settings,
  updateSettings,
  audioDevices,
  refreshingDevices,
  handleRefreshDevices,
}: AudioTabProps) {
  return (
    <section
      id="settings-panel-audio"
      role="tabpanel"
      aria-labelledby="settings-tab-audio"
      className="animate-slide-up"
    >
      <SectionHeader
        title="Audio Input"
        subtitle="Select your microphone input device"
      />
      <DropdownSelect
        id="select-input-device"
        label="Input Device"
        description="Choose which microphone to use for recording"
        value={settings.input_device}
        options={
          audioDevices.length > 0
            ? audioDevices.map((d) => ({
                value: d.name,
                label: d.is_default ? `${d.name} (Default)` : d.name,
              }))
            : [{ value: 'default', label: 'System Default' }]
        }
        onChange={(v) => updateSettings({ input_device: v })}
      />
      <button
        type="button"
        onClick={handleRefreshDevices}
        disabled={refreshingDevices}
        className="
          mt-2 inline-flex items-center gap-2 px-3 py-1.5 rounded-lg
          text-xs font-medium text-text-secondary
          bg-surface-alt border border-border
          hover:bg-surface-hover hover:text-text-primary
          disabled:opacity-60 disabled:cursor-not-allowed
          transition-colors duration-150
          focus-visible:outline-2 focus-visible:outline-offset-2
          focus-visible:outline-primary cursor-pointer
        "
      >
        <svg 
          className={`w-3.5 h-3.5 ${refreshingDevices ? 'animate-spin text-primary' : ''}`}
          fill="none" 
          stroke="currentColor" 
          viewBox="0 0 24 24"
        >
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M16.023 9.348h4.992v-.001M2.985 19.644v-4.992m0 0h4.992m-4.993 0l3.181 3.183a8.25 8.25 0 0013.803-3.7M4.031 9.865a8.25 8.25 0 0113.803-3.7l3.181 3.182" />
        </svg>
        {refreshingDevices ? 'Scanning...' : 'Refresh Devices'}
      </button>

      <MicrophoneActivityTest deviceName={settings.input_device} />
    </section>
  );
}
