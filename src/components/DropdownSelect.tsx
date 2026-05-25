interface DropdownSelectProps {
  id: string;
  label: string;
  description?: string;
  value: string;
  options: { value: string; label: string }[];
  onChange: (value: string) => void;
}

export function DropdownSelect({
  id,
  label,
  description,
  value,
  options,
  onChange,
}: DropdownSelectProps) {
  const hasValue = options.some((opt) => opt.value === value);
  const selectOptions = hasValue
    ? options
    : value
    ? [...options, { value, label: `${value} (configured)` }]
    : options;

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
      <div className="relative">
        <select
          id={id}
          value={value}
          onChange={(e) => onChange(e.target.value)}
          style={{ colorScheme: 'dark' }}
          className="
            w-full appearance-none rounded-lg bg-surface-alt border border-border
            px-3 py-2 pr-10 text-sm text-text-primary
            transition-colors duration-150
            hover:border-primary/50 focus:border-primary focus:outline-none
            focus:ring-1 focus:ring-primary/40
            cursor-pointer
          "
        >
          {selectOptions.map((opt) => (
            <option key={opt.value} value={opt.value}>
              {opt.label}
            </option>
          ))}
        </select>
        <div className="pointer-events-none absolute inset-y-0 right-0 flex items-center pr-3">
          <svg
            className="h-4 w-4 text-text-muted"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
            aria-hidden="true"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M19 9l-7 7-7-7"
            />
          </svg>
        </div>
      </div>
    </div>
  );
}
