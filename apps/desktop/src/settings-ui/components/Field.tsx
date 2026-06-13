/**
 * Layout primitives for settings rows. `Row` is a labelled control line;
 * `Section` groups rows under a heading. Both are pure layout (no theme
 * decisions beyond semantic tokens) so panels read declaratively.
 */
import type { ReactNode } from "react";

export function Section({
  title,
  description,
  children,
}: {
  title: string;
  description?: string;
  children: ReactNode;
}) {
  return (
    <section className="mb-8">
      <div className="mb-3">
        <h3 className="font-display text-[13px] font-semibold uppercase tracking-[0.14em] text-cf-faint">
          {title}
        </h3>
        {description && (
          <p className="mt-1 text-[13px] leading-relaxed text-cf-muted">
            {description}
          </p>
        )}
      </div>
      <div className="overflow-hidden rounded-xl border border-cf-border bg-cf-elevated">
        {children}
      </div>
    </section>
  );
}

export function Row({
  label,
  hint,
  control,
  htmlFor,
}: {
  label: string;
  hint?: ReactNode;
  control: ReactNode;
  htmlFor?: string;
}) {
  return (
    <div className="flex items-center justify-between gap-6 border-b border-cf-border px-4 py-3.5 last:border-b-0">
      <div className="min-w-0">
        <label
          htmlFor={htmlFor}
          className="block text-[14px] font-medium text-cf-text"
        >
          {label}
        </label>
        {hint && (
          <p className="mt-0.5 text-[12.5px] leading-relaxed text-cf-muted">
            {hint}
          </p>
        )}
      </div>
      <div className="shrink-0">{control}</div>
    </div>
  );
}

/** Themed native select with a consistent chrome. */
export function Select({
  value,
  onChange,
  options,
  id,
  disabled,
}: {
  value: string;
  onChange: (v: string) => void;
  options: { value: string; label: string }[];
  id?: string;
  disabled?: boolean;
}) {
  return (
    <div className="relative">
      <select
        id={id}
        value={value}
        disabled={disabled}
        onChange={(e) => onChange(e.target.value)}
        className="appearance-none rounded-lg border border-cf-border-strong bg-cf-inset
          py-2 pl-3 pr-9 text-[13.5px] text-cf-text outline-none transition-colors
          hover:border-cf-accent focus-visible:border-cf-accent
          focus-visible:ring-2 focus-visible:ring-cf-accent/40
          disabled:opacity-40 disabled:cursor-not-allowed cursor-pointer"
      >
        {options.map((o) => (
          <option key={o.value} value={o.value} className="bg-cf-elevated text-cf-text">
            {o.label}
          </option>
        ))}
      </select>
      <svg
        className="pointer-events-none absolute right-2.5 top-1/2 -translate-y-1/2 text-cf-muted"
        width="14" height="14" viewBox="0 0 24 24" fill="none"
        stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round"
      >
        <path d="m6 9 6 6 6-6" />
      </svg>
    </div>
  );
}

/** Themed text input. */
export function TextInput({
  value,
  onChange,
  placeholder,
  type = "text",
  id,
  disabled,
  mono,
}: {
  value: string;
  onChange: (v: string) => void;
  placeholder?: string;
  type?: "text" | "password";
  id?: string;
  disabled?: boolean;
  mono?: boolean;
}) {
  return (
    <input
      id={id}
      type={type}
      value={value}
      disabled={disabled}
      placeholder={placeholder}
      spellCheck={false}
      autoComplete="off"
      onChange={(e) => onChange(e.target.value)}
      className={`w-[260px] rounded-lg border border-cf-border-strong bg-cf-inset px-3 py-2
        text-[13.5px] text-cf-text outline-none transition-colors
        placeholder:text-cf-faint hover:border-cf-accent
        focus-visible:border-cf-accent focus-visible:ring-2 focus-visible:ring-cf-accent/40
        disabled:opacity-40 disabled:cursor-not-allowed ${mono ? "font-mono" : ""}`}
    />
  );
}
