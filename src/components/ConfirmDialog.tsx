import { useEffect, useRef, useCallback, useId } from 'react';

interface ConfirmDialogProps {
  open: boolean;
  title: string;
  message: string;
  onConfirm: () => void;
  onCancel: () => void;
  confirmLabel?: string;
  danger?: boolean;
}

export function ConfirmDialog({
  open,
  title,
  message,
  onConfirm,
  onCancel,
  confirmLabel = 'Confirm',
  danger = false,
}: ConfirmDialogProps) {
  const cancelRef = useRef<HTMLButtonElement>(null);
  const dialogRef = useRef<HTMLDivElement>(null);
  const dialogId = useId();

  // Focus cancel button on open
  useEffect(() => {
    if (open) {
      cancelRef.current?.focus();
    }
  }, [open]);

  // Escape closes dialog + Tab focus trap
  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      if (e.key === 'Escape') {
        e.stopPropagation();
        onCancel();
      }
      if (e.key === 'Tab' && dialogRef.current) {
        const focusable = dialogRef.current.querySelectorAll<HTMLElement>(
          'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])'
        );
        const first = focusable[0];
        const last = focusable[focusable.length - 1];
        if (e.shiftKey) {
          if (document.activeElement === first) {
            e.preventDefault();
            last?.focus();
          }
        } else {
          if (document.activeElement === last) {
            e.preventDefault();
            first?.focus();
          }
        }
      }
    },
    [onCancel],
  );

  if (!open) return null;

  return (
    <div
      className="
        fixed inset-0 z-50 flex items-center justify-center
        bg-black/60 backdrop-blur-sm
        animate-fade-in
      "
      onClick={onCancel}
      onKeyDown={handleKeyDown}
      role="dialog"
      aria-modal="true"
      aria-labelledby={`${dialogId}-title`}
      aria-describedby={`${dialogId}-message`}
    >
      <div
        ref={dialogRef}
        className="
          relative bg-surface border border-border rounded-xl
          shadow-2xl shadow-black/40 p-6 max-w-sm w-full mx-4
          animate-zoom-in
        "
        onClick={(e) => e.stopPropagation()}
      >
        <h3
          id={`${dialogId}-title`}
          className="text-lg font-semibold text-text-primary mb-2"
        >
          {title}
        </h3>
        <p
          id={`${dialogId}-message`}
          className="text-sm text-text-secondary mb-6 leading-relaxed"
        >
          {message}
        </p>
        <div className="flex items-center justify-end gap-3">
          <button
            ref={cancelRef}
            type="button"
            onClick={onCancel}
            className="
              px-4 py-2 rounded-lg text-sm font-medium
              text-text-secondary bg-surface-alt border border-border
              hover:bg-surface-hover hover:text-text-primary
              transition-colors duration-150
              focus-visible:outline-2 focus-visible:outline-offset-2
              focus-visible:outline-primary
            "
          >
            Cancel
          </button>
          <button
            type="button"
            onClick={onConfirm}
            className={`
              px-4 py-2 rounded-lg text-sm font-medium text-white
              transition-colors duration-150
              focus-visible:outline-2 focus-visible:outline-offset-2
              ${
                danger
                  ? 'bg-danger hover:bg-danger/80 focus-visible:outline-danger'
                  : 'bg-primary hover:bg-primary-hover focus-visible:outline-primary'
              }
            `}
          >
            {confirmLabel}
          </button>
        </div>
      </div>
    </div>
  );
}

