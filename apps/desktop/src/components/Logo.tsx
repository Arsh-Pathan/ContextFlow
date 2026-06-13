import { SVGProps, useId } from "react";

export function Logo(props: SVGProps<SVGSVGElement>) {
  const gradientId = useId();
  return (
    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 64 64" {...props}>
      <defs>
        <linearGradient id={gradientId} x1="0" y1="0" x2="1" y2="1">
          <stop offset="0%" stopColor="var(--logo-color-1, var(--cf-accent, #10b981))" />
          <stop offset="100%" stopColor="var(--logo-color-2, var(--cf-accent-2, #0e7490))" />
        </linearGradient>
      </defs>
      <circle cx="32" cy="32" r="28" fill={`url(#${gradientId})`} />
      <circle cx="32" cy="32" r="8" fill="var(--cf-text, white)" fillOpacity="0.85" />
    </svg>
  );
}
