import type { AppSettings } from '../../types';
import { TextArea } from '../../components/TextArea';
import { SectionHeader } from '../../components/SectionHeader';

interface PromptsTabProps {
  settings: AppSettings;
  updateSettings: (patch: Partial<AppSettings>) => void;
  defaultPrompts: { stt_cleanup_prompt: string; command_mode_prompt: string } | null;
}

export function PromptsTab({ settings, updateSettings, defaultPrompts }: PromptsTabProps) {
  return (
    <section
      id="settings-panel-prompts"
      role="tabpanel"
      aria-labelledby="settings-tab-prompts"
      className="animate-slide-up"
    >
      <SectionHeader
        title="Prompts"
        subtitle="Customize system instructions for voice flows"
      />
      
      <div className="mb-6">
        <TextArea
          id="textarea-stt-prompt"
          label="STT Cleanup Prompt"
          description="System prompt for cleaning up speech-to-text output. Reverts to default if cleared."
          value={settings.stt_cleanup_prompt}
          onChange={(v) => updateSettings({ stt_cleanup_prompt: v })}
          rows={5}
        />
        <button
          type="button"
          onClick={() => {
            if (defaultPrompts) {
              updateSettings({ stt_cleanup_prompt: defaultPrompts.stt_cleanup_prompt });
            }
          }}
          className="mt-1 text-xs font-semibold text-primary hover:text-primary-hover hover:underline cursor-pointer focus:outline-none"
        >
          Reset to Default Prompt
        </button>
      </div>

      <div className="mb-6">
        <TextArea
          id="textarea-command-prompt"
          label="Command Mode Prompt"
          description="System prompt for command mode. The LLM will interpret spoken commands using this context."
          value={settings.command_mode_prompt}
          onChange={(v) => updateSettings({ command_mode_prompt: v })}
          rows={5}
        />
        <button
          type="button"
          onClick={() => {
            if (defaultPrompts) {
              updateSettings({ command_mode_prompt: defaultPrompts.command_mode_prompt });
            }
          }}
          className="mt-1 text-xs font-semibold text-primary hover:text-primary-hover hover:underline cursor-pointer focus:outline-none"
        >
          Reset to Default Prompt
        </button>
      </div>
    </section>
  );
}
