import { useState, useCallback, useEffect, useRef } from 'react';

interface KeyCaptureProps {
  id: string;
  label: string;
  value: string;
  onChange: (keys: string) => void;
}

function parseHotkey(hotkey: string): string[] {
  if (!hotkey) return [];
  return hotkey.split('+').map((k) => k.trim()).filter(Boolean);
}

function formatKeyEvent(e: KeyboardEvent): string | null {
  const parts: string[] = [];

  if (e.ctrlKey) parts.push('Ctrl');
  if (e.altKey) parts.push('Alt');
  if (e.shiftKey) parts.push('Shift');
  if (e.metaKey) parts.push('Super');

  const key = e.key;
  // Skip if only a modifier key was pressed
  const modifierKeys = ['Control', 'Alt', 'Shift', 'Meta'];
  if (modifierKeys.includes(key)) return null;

  // Normalize key names
  const keyMap: Record<string, string> = {
    ' ': 'Space',
    ArrowUp: 'Up',
    ArrowDown: 'Down',
    ArrowLeft: 'Left',
    ArrowRight: 'Right',
    CapsLock: 'CapsLock',
    Escape: 'Escape',
    Enter: 'Enter',
    Backspace: 'Backspace',
    Delete: 'Delete',
    Tab: 'Tab',
    Home: 'Home',
    End: 'End',
    PageUp: 'PageUp',
    PageDown: 'PageDown',
    Insert: 'Insert',
  };

  const normalizedKey = keyMap[key] ?? (key.length === 1 ? key.toUpperCase() : key);
  parts.push(normalizedKey);

  return parts.join('+');
}

export function KeyCapture({ id, label, value, onChange }: KeyCaptureProps) {
  const [listening, setListening] = useState(false);
  const containerRef = useRef<HTMLDivElement>(null);
  const keys = parseHotkey(value);

  const startListening = useCallback(() => {
    setListening(true);
  }, []);

  const stopListening = useCallback(() => {
    setListening(false);
  }, []);

  // Global keydown listener when in capture mode
  useEffect(() => {
    if (!listening) return;

    const handleKeyDown = (e: KeyboardEvent) => {
      e.preventDefault();
      e.stopPropagation();

      if (e.key === 'Escape') {
        stopListening();
        return;
      }

      const combo = formatKeyEvent(e);
      if (combo) {
        onChange(combo);
        stopListening();
      }
    };

    const handleClickOutside = (e: MouseEvent) => {
      if (containerRef.current && !containerRef.current.contains(e.target as Node)) {
        stopListening();
      }
    };

    window.addEventListener('keydown', handleKeyDown, true);
    window.addEventListener('mousedown', handleClickOutside, true);

    return () => {
      window.removeEventListener('keydown', handleKeyDown, true);
      window.removeEventListener('mousedown', handleClickOutside, true);
    };
  }, [listening, onChange, stopListening]);

  return (
    <div className="py-3" ref={containerRef}>
      <label
        htmlFor={id}
        className="block text-sm font-medium text-text-primary mb-2"
      >
        {label}
      </label>
      <div className="flex gap-2">
        <button
          id={id}
          type="button"
          onClick={startListening}
          onKeyDown={(e) => {
            if (e.key === 'Enter' || e.key === ' ') {
              e.preventDefault();
              startListening();
            }
          }}
          className={`
            flex-1 flex items-center gap-2 rounded-lg border px-3 py-2.5
            text-sm transition-all duration-200 min-h-[42px]
            focus-visible:outline-2 focus-visible:outline-offset-2
            focus-visible:outline-primary cursor-pointer
            ${
              listening
                ? 'border-primary bg-primary/10 animate-pulse'
                : 'border-border bg-surface-alt hover:border-primary/50'
            }
          `}
          aria-label={`${label}: ${value || 'Not set'}. Click to change.`}
        >
          {listening ? (
            <span className="text-primary text-sm italic">Press keys…</span>
          ) : keys.length > 0 ? (
            <div className="flex items-center gap-1.5 flex-wrap">
              {keys.map((key, i) => (
                <span key={i}>
                  {i > 0 && (
                    <span className="text-text-muted mx-0.5">+</span>
                  )}
                  <kbd
                    className="
                      inline-flex items-center justify-center
                      rounded-md border border-border bg-surface
                      px-2 py-0.5 text-xs font-mono font-medium
                      text-text-primary shadow-sm min-w-[28px]
                    "
                  >
                    {key}
                  </kbd>
                </span>
              ))}
            </div>
          ) : (
            <span className="text-text-muted text-sm">Click to set hotkey</span>
          )}
        </button>
        {value && !listening && (
          <button
            type="button"
            onClick={() => onChange('')}
            title="Clear Hotkey"
            className="
              px-3.5 rounded-lg border border-border bg-surface-alt text-text-secondary
              hover:text-danger hover:border-danger/30 hover:bg-danger/10
              transition-all duration-150 cursor-pointer
              focus-visible:outline-2 focus-visible:outline-offset-2
              focus-visible:outline-danger
            "
          >
            <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        )}
      </div>
    </div>
  );
}
