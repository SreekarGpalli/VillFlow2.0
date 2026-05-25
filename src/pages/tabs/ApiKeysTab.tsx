import type { AppSettings } from '../../types';
import { PasswordField } from '../../components/PasswordField';
import { DropdownSelect } from '../../components/DropdownSelect';
import { SectionHeader } from '../../components/SectionHeader';

interface ApiKeysTabProps {
  settings: AppSettings;
  updateSettings: (patch: Partial<AppSettings>) => void;
  speechmaticsKey: string;
  speechmaticsSaveStatus: 'idle' | 'saving' | 'saved' | 'error';
  onChangeSpeechmaticsKey: (val: string) => void;
  groqKey: string;
  groqSaveStatus: 'idle' | 'saving' | 'saved' | 'error';
  onChangeGroqKey: (val: string) => void;
  groqModels: string[];
  fallbackModels: string[];
}

export function ApiKeysTab({
  settings,
  updateSettings,
  speechmaticsKey,
  speechmaticsSaveStatus,
  onChangeSpeechmaticsKey,
  groqKey,
  groqSaveStatus,
  onChangeGroqKey,
  groqModels,
  fallbackModels,
}: ApiKeysTabProps) {
  return (
    <section
      id="settings-panel-apikeys"
      role="tabpanel"
      aria-labelledby="settings-tab-apikeys"
      className="animate-slide-up"
    >
      <SectionHeader
        title="Services & APIs"
        subtitle="Configure API credentials and service settings"
      />

      <div className="mb-6">
        <h3 className="text-sm font-semibold text-text-primary mb-1 flex items-center gap-2">
          <span className="w-2 h-2 rounded-full bg-secondary" />
          Speechmatics (Speech-to-Text)
        </h3>
        <p className="text-xs text-text-muted mb-3">
          Real-time speech-to-text transcription service
        </p>
        <PasswordField
          id="input-speechmatics-key"
          label="API Key"
          value={speechmaticsKey}
          saveStatus={speechmaticsSaveStatus}
          onChange={onChangeSpeechmaticsKey}
          placeholder="Enter Speechmatics API key…"
        />
        <DropdownSelect
          id="select-speechmatics-region"
          label="Region"
          value={settings.speechmatics_region}
          options={[
            { value: 'eu', label: 'Europe (eu2)' },
            { value: 'usa', label: 'United States (usa)' },
            { value: 'au', label: 'Australia (au)' },
          ]}
          onChange={(v) => updateSettings({ speechmatics_region: v as 'eu' | 'usa' | 'au' })}
        />
        <DropdownSelect
          id="select-speechmatics-operating-point"
          label="Operating Point"
          description="Higher accuracy uses more processing time"
          value={settings.speechmatics_operating_point}
          options={[
            { value: 'enhanced', label: 'Enhanced (recommended)' },
            { value: 'standard', label: 'Standard' },
          ]}
          onChange={(v) =>
            updateSettings({ speechmatics_operating_point: v as 'standard' | 'enhanced' })
          }
        />
        <DropdownSelect
          id="select-language"
          label="Spoken Language"
          description="The language spoken during dictation"
          value={settings.language}
          options={[
            { value: 'en', label: 'English' },
            { value: 'de', label: 'German' },
            { value: 'fr', label: 'French' },
            { value: 'es', label: 'Spanish' },
            { value: 'it', label: 'Italian' },
            { value: 'pt', label: 'Portuguese' },
            { value: 'nl', label: 'Dutch' },
            { value: 'ja', label: 'Japanese' },
            { value: 'ko', label: 'Korean' },
            { value: 'zh', label: 'Chinese (Mandarin)' },
            { value: 'ar', label: 'Arabic' },
            { value: 'hi', label: 'Hindi' },
            { value: 'ru', label: 'Russian' },
            { value: 'pl', label: 'Polish' },
            { value: 'tr', label: 'Turkish' },
          ]}
          onChange={(v) => updateSettings({ language: v })}
        />
      </div>

      <div className="pt-4 border-t border-border/50">
        <h3 className="text-sm font-semibold text-text-primary mb-1 flex items-center gap-2">
          <span className="w-2 h-2 rounded-full bg-primary" />
          Groq (Language Model)
        </h3>
        <p className="text-xs text-text-muted mb-3">
          LLM inference for text cleanup and commands
        </p>
        <PasswordField
          id="input-groq-key"
          label="API Key"
          value={groqKey}
          saveStatus={groqSaveStatus}
          onChange={onChangeGroqKey}
          placeholder="Enter Groq API key…"
        />
        <DropdownSelect
          id="select-llm-model"
          label="Model"
          description="The Groq model used for text cleanup and command processing"
          value={settings.llm_model}
          options={
            groqModels.length > 0
              ? groqModels.map((m) => ({ value: m, label: m }))
              : fallbackModels.map((m) => ({ value: m, label: m }))
          }
          onChange={(v) => updateSettings({ llm_model: v })}
        />
        <div className="mt-4 grid grid-cols-2 gap-4 pb-4">
          <div>
            <label className="block text-xs font-semibold text-text-primary mb-1">
              Temperature ({settings.llm_temperature})
            </label>
            <input
              type="range"
              min="0"
              max="2"
              step="0.1"
              className="w-full h-1.5 bg-surface-alt rounded-lg appearance-none cursor-pointer accent-primary"
              value={settings.llm_temperature}
              onChange={(e) => updateSettings({ llm_temperature: parseFloat(e.target.value) })}
            />
            <p className="text-[10px] text-text-muted mt-1">
              Controls randomness: 0 is precise, 2 is creative.
            </p>
          </div>
          <div>
            <label className="block text-xs font-semibold text-text-primary mb-1">
              Max Tokens
            </label>
            <input
              type="number"
              className="w-full px-3 py-2 rounded-lg bg-surface border border-border text-xs text-text-primary focus:outline-none focus:border-primary"
              value={settings.llm_max_tokens}
              onChange={(e) => updateSettings({ llm_max_tokens: parseInt(e.target.value, 10) || 2048 })}
            />
            <p className="text-[10px] text-text-muted mt-1">
              Maximum length of the generated response.
            </p>
          </div>
        </div>
      </div>
    </section>
  );
}
