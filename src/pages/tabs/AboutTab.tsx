import { SectionHeader } from '../../components/SectionHeader';
import { Logo } from '../../components/Logo';

interface AboutTabProps {
  appVersion: string;
  configPath: string;
  handleOpenLogFolder: () => Promise<void>;
  setShowResetConfirm: (v: boolean) => void;
}

export function AboutTab({
  appVersion,
  configPath,
  handleOpenLogFolder,
  setShowResetConfirm,
}: AboutTabProps) {
  return (
    <section
      id="settings-panel-about"
      role="tabpanel"
      aria-labelledby="settings-tab-about"
      className="animate-slide-up"
    >
      <SectionHeader
        title="About & System"
        subtitle="VillFlow 2.0 — Voice-to-text reimagined"
      />

      {/* App info */}
      <div className="rounded-xl bg-surface-alt border border-border p-5 mb-6">
        <div className="flex items-center gap-4 mb-4">
          <Logo size={48} />
          <div>
            <h3 className="text-base font-semibold text-text-primary">
              VillFlow
            </h3>
            <p className="text-sm text-text-muted">Version {appVersion}</p>
          </div>
        </div>
        <p className="text-sm text-text-secondary leading-relaxed">
          A lightweight, global push-to-talk voice assistant for Windows.
          Uses Speechmatics for real-time STT and Groq for intelligent
          text cleanup and command processing.
        </p>
      </div>

      {/* Config path */}
      <div className="mb-4">
        <label className="block text-sm font-medium text-text-primary mb-1.5">
          Config File Location
        </label>
        <div
          className="
            w-full rounded-lg bg-surface-alt border border-border
            px-3 py-2 text-xs text-text-muted font-mono
            select-text cursor-text break-all
          "
        >
          {configPath || 'Loading…'}
        </div>
      </div>

      {/* Actions */}
      <div className="flex flex-col gap-3 mt-6">
        <button
          type="button"
          onClick={handleOpenLogFolder}
          className="
            inline-flex items-center justify-center gap-2
            px-4 py-2.5 rounded-lg text-sm font-medium
            text-text-primary bg-surface-alt border border-border
            hover:bg-surface-hover transition-colors duration-150
            focus-visible:outline-2 focus-visible:outline-offset-2
            focus-visible:outline-primary cursor-pointer
          "
        >
          <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M2.25 12.75V12A2.25 2.25 0 014.5 9.75h15A2.25 2.25 0 0121.75 12v.75m-8.69-6.44l-2.12-2.12a1.5 1.5 0 00-1.061-.44H4.5A2.25 2.25 0 002.25 6v12a2.25 2.25 0 002.25 2.25h15A2.25 2.25 0 0021.75 18V9a2.25 2.25 0 00-2.25-2.25h-5.379a1.5 1.5 0 01-1.06-.44z" />
          </svg>
          Open Log Folder
        </button>
        <button
          type="button"
          onClick={() => setShowResetConfirm(true)}
          className="
            inline-flex items-center justify-center gap-2
            px-4 py-2.5 rounded-lg text-sm font-medium
            text-danger bg-danger/10 border border-danger/30
            hover:bg-danger/20 transition-colors duration-150
            focus-visible:outline-2 focus-visible:outline-offset-2
            focus-visible:outline-danger cursor-pointer
          "
        >
          <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M16.023 9.348h4.992v-.001M2.985 19.644v-4.992m0 0h4.992m-4.993 0l3.181 3.183a8.25 8.25 0 0013.803-3.7M4.031 9.865a8.25 8.25 0 0113.803-3.7l3.181 3.182" />
          </svg>
          Reset All Settings
        </button>
      </div>
    </section>
  );
}
