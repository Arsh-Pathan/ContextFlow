/** Accessible themed switch. Colour follows the active theme's accent. */
interface ToggleProps {
  checked: boolean;
  onChange: (next: boolean) => void;
  label?: string;
  id?: string;
  disabled?: boolean;
}

export function Toggle({ checked, onChange, label, id, disabled }: ToggleProps) {
  return (
    <button
      type="button"
      role="switch"
      aria-checked={checked}
      aria-label={label}
      id={id}
      disabled={disabled}
      onClick={() => onChange(!checked)}
      className={`relative inline-flex h-[24px] w-[42px] shrink-0 items-center rounded-full
        transition-colors duration-300 ease-out outline-none
        focus-visible:ring-2 focus-visible:ring-cf-accent focus-visible:ring-offset-2
        focus-visible:ring-offset-cf-bg
        ${disabled ? "opacity-40 cursor-not-allowed" : "cursor-pointer"}`}
      style={{
        background: checked ? "var(--cf-accent)" : "var(--cf-bg-inset)",
        boxShadow: checked
          ? "0 0 12px rgba(var(--cf-glow-rgb), 0.5), inset 0 0 0 1px var(--cf-accent)"
          : "inset 0 0 0 1px var(--cf-border-strong)",
      }}
    >
      <span
        className="inline-block h-[18px] w-[18px] rounded-full bg-white shadow-sm
          transition-transform duration-300 ease-[cubic-bezier(0.34,1.56,0.64,1)]"
        style={{
          transform: checked ? "translateX(21px)" : "translateX(3px)",
        }}
      />
    </button>
  );
}
