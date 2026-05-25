interface TextAreaProps {
  id: string;
  label: string;
  description?: string;
  value: string;
  onChange: (value: string) => void;
  rows?: number;
}

export function TextArea({
  id,
  label,
  description,
  value,
  onChange,
  rows = 4,
}: TextAreaProps) {
  return (
    <div className="py-3">
      <label
        htmlFor={id}
        className="block text-sm font-medium text-text-primary mb-1"
      >
        {label}
      </label>
      {description && (
        <p className="text-xs text-text-muted mb-2 leading-relaxed">
          {description}
        </p>
      )}
      <textarea
        id={id}
        value={value}
        onChange={(e) => onChange(e.target.value)}
        rows={rows}
        spellCheck={false}
        className="
          w-full rounded-lg bg-surface-alt border border-border
          px-3 py-2 text-sm text-text-primary font-mono leading-relaxed
          placeholder:text-text-muted resize-y min-h-[80px]
          transition-colors duration-150
          hover:border-primary/50 focus:border-primary focus:outline-none
          focus:ring-1 focus:ring-primary/40
        "
      />
    </div>
  );
}
