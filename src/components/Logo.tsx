import React from 'react';

interface LogoProps {
  /** Width and height of the logo container in pixels */
  size?: number;
  /** Additional CSS classes */
  className?: string;
  /** Whether to show the full brand name next to the logo */
  showBrand?: boolean;
  /** Brand text size class (tailwind) */
  brandClass?: string;
}

/**
 * VillFlow "V2" logo — a clean, modern text-mark rendered as inline SVG.
 *
 * The "V" uses a bold gradient fill (primary → secondary) while the "2"
 * is rendered in a lighter weight for visual contrast.
 */
export const Logo: React.FC<LogoProps> = ({
  size = 36,
  className = '',
  showBrand = false,
  brandClass = 'text-lg font-bold text-text-primary tracking-tight',
}) => {
  const id = React.useId();
  return (
    <span className={`inline-flex items-center gap-2.5 ${className}`}>
      <svg
        width={size}
        height={size}
        viewBox="0 0 48 48"
        fill="none"
        xmlns="http://www.w3.org/2000/svg"
        aria-label="VillFlow V2 logo"
        role="img"
      >
        {/* Background rounded square */}
        <rect
          x="2"
          y="2"
          width="44"
          height="44"
          rx="12"
          fill={`url(#${id}-bg)`}
        />

        {/* Gradient definitions */}
        <defs>
          <linearGradient
            id={`${id}-bg`}
            x1="2"
            y1="2"
            x2="46"
            y2="46"
            gradientUnits="userSpaceOnUse"
          >
            <stop offset="0%" stopColor="#6366f1" />
            <stop offset="100%" stopColor="#4f46e5" />
          </linearGradient>
          <linearGradient
            id={`${id}-text`}
            x1="8"
            y1="12"
            x2="40"
            y2="40"
            gradientUnits="userSpaceOnUse"
          >
            <stop offset="0%" stopColor="#ffffff" />
            <stop offset="100%" stopColor="#c7d2fe" />
          </linearGradient>
        </defs>

        {/* "V2" text */}
        <text
          x="24"
          y="34"
          textAnchor="middle"
          fill={`url(#${id}-text)`}
          fontFamily="Inter, system-ui, sans-serif"
          fontSize="26"
          fontWeight="800"
          letterSpacing="-1"
        >
          V2
        </text>

        {/* Subtle inner border / shine */}
        <rect
          x="3"
          y="3"
          width="42"
          height="42"
          rx="11"
          stroke="white"
          strokeOpacity="0.12"
          strokeWidth="1"
          fill="none"
        />
      </svg>
      {showBrand && (
        <span className={brandClass}>VillFlow</span>
      )}
    </span>
  );
};
