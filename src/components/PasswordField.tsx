import { useState, useCallback } from 'react';

interface PasswordFieldProps {
  id: string;
  label: string;
  value: string;
  onChange: (value: string) => void;
  placeholder?: string;
  saveStatus?: 'idle' | 'saving' | 'saved' | 'error';
}

export function PasswordField({
  id,
  label,
  value,
  onChange,
  placeholder = 'Enter API key…',
  saveStatus = 'idle',
}: PasswordFieldProps) {
  const [visible, setVisible] = useState(false);

  const toggleVisibility = useCallback(() => {
    setVisible((v) => !v);
  }, []);

  return (
    <div className="py-3">
      <div className="flex items-center justify-between mb-1">
        <label
          htmlFor={id}
          className="block text-sm font-medium text-text-primary cursor-pointer"
        >
          {label}
        </label>
        {saveStatus !== 'idle' && (
          <span className={`text-xs flex items-center gap-1 font-medium animate-fade-in ${
            saveStatus === 'saving' ? 'text-text-muted' :
            saveStatus === 'saved' ? 'text-success' :
            'text-danger'
          }`}>
            {saveStatus === 'saving' && <div className="w-2.5 h-2.5 border-2 border-current border-t-transparent rounded-full animate-spin" />}
            {saveStatus === 'saved' && 'Saved'}
            {saveStatus === 'error' && 'Save failed'}
          </span>
        )}
      </div>
      <div className="relative">
        <input
          id={id}
          type={visible ? 'text' : 'password'}
          value={value}
          onChange={(e) => onChange(e.target.value)}
          placeholder={placeholder}
          autoComplete="off"
          spellCheck={false}
          className="
            w-full rounded-lg bg-surface-alt border border-border
            px-3 py-2 pr-10 text-sm text-text-primary font-mono
            placeholder:text-text-muted
            transition-colors duration-150
            hover:border-primary/50 focus:border-primary focus:outline-none
            focus:ring-1 focus:ring-primary/40
          "
        />
        <button
          type="button"
          onClick={toggleVisibility}
          tabIndex={-1}
          aria-label={visible ? 'Hide value' : 'Show value'}
          className="
            absolute inset-y-0 right-0 flex items-center pr-3
            text-text-muted hover:text-text-secondary transition-colors
          "
        >
          {visible ? (
            /* Eye-off icon */
            <svg
              className="h-4 w-4"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
              aria-hidden="true"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M13.875 18.825A10.05 10.05 0 0112 19c-4.478 0-8.268-2.943-9.543-7a9.97 9.97 0 011.563-3.029m5.858.908a3 3 0 114.243 4.243M9.878 9.878l4.242 4.242M9.878 9.878L3 3m6.878 6.878L21 21"
              />
            </svg>
          ) : (
            /* Eye icon */
            <svg
              className="h-4 w-4"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
              aria-hidden="true"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"
              />
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z"
              />
            </svg>
          )}
        </button>
      </div>
    </div>
  );
}
