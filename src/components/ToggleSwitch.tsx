import { useCallback } from 'react';

interface ToggleSwitchProps {
  id: string;
  label: string;
  description?: string;
  checked: boolean;
  onChange: (checked: boolean) => void;
}

export function ToggleSwitch({
  id,
  label,
  description,
  checked,
  onChange,
}: ToggleSwitchProps) {
  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      if (e.key === 'Enter' || e.key === ' ') {
        e.preventDefault();
        onChange(!checked);
      }
    },
    [checked, onChange],
  );

  return (
    <div className="flex items-center justify-between py-3 group">
      <div className="flex-1 min-w-0 pr-4">
        <label
          htmlFor={id}
          className="text-sm font-medium text-text-primary cursor-pointer select-none"
        >
          {label}
        </label>
        {description && (
          <p className="mt-0.5 text-xs text-text-muted leading-relaxed">
            {description}
          </p>
        )}
      </div>
      <button
        id={id}
        type="button"
        role="switch"
        aria-checked={checked}
        aria-label={label}
        tabIndex={0}
        onClick={() => onChange(!checked)}
        onKeyDown={handleKeyDown}
        className={`
          relative inline-flex h-6 w-11 shrink-0 cursor-pointer items-center
          rounded-full border-2 border-transparent transition-colors duration-200
          ease-in-out focus-visible:outline-2 focus-visible:outline-offset-2
          focus-visible:outline-primary
          ${checked ? 'bg-primary' : 'bg-border'}
        `}
      >
        <span
          aria-hidden="true"
          className={`
            pointer-events-none inline-block h-4 w-4 transform rounded-full
            bg-white shadow-sm ring-0 transition-transform duration-200 ease-in-out
            ${checked ? 'translate-x-[22px]' : 'translate-x-0.5'}
          `}
        />
      </button>
    </div>
  );
}
