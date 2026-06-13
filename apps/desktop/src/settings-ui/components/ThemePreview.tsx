/**
 * A self-contained live preview of a theme: a mini canvas showing the bubble's
 * aurora ring and visualizer dots, painted with the theme's own tokens. The
 * variables are scoped to this node via inline style, so many previews with
 * different themes can render at once on the real settings page.
 *
 * This is *preview chrome*, deliberately a simplified echo of the real bubble —
 * enough to read colour + motion at a glance without importing the full Bubble.
 */
import { useMemo } from "react";

import { themeStyleVars, type Theme } from "../../theme";

export function ThemePreview({
  theme,
  active,
  onSelect,
}: {
  theme: Theme;
  active: boolean;
  onSelect: () => void;
}) {
  const vars = useMemo(() => themeStyleVars(theme), [theme]);

  return (
    <button
      type="button"
      onClick={onSelect}
      aria-pressed={active}
      data-cf-motion={theme.motion}
      style={vars}
      className={`group relative flex flex-col overflow-hidden rounded-xl border text-left
        transition-all duration-300 ease-out outline-none
        focus-visible:ring-2 focus-visible:ring-cf-accent
        ${active
          ? "border-cf-accent shadow-[0_0_0_1px_var(--cf-accent),0_8px_28px_rgba(var(--cf-glow-rgb),0.28)]"
          : "border-cf-border hover:border-cf-border-strong hover:-translate-y-0.5"}`}
    >
      {/* Mini canvas painted in the theme's own bg. */}
      <div
        className="relative h-[88px] w-full overflow-hidden"
        style={{ background: "var(--cf-bg)" }}
      >
        {/* faint elevated card to show surface contrast */}
        <div
          className="absolute inset-x-3 top-3 h-7 rounded-md"
          style={{ background: "var(--cf-bg-elevated)", border: "1px solid var(--cf-border)" }}
        />
        {/* mini bubble */}
        <div className="absolute bottom-3 left-1/2 flex h-8 -translate-x-1/2 items-center gap-2 rounded-full px-2.5"
          style={{ background: "var(--cf-bubble-surface)", boxShadow: "0 0 16px rgba(var(--cf-glow-rgb),0.45)" }}
        >
          <span className="h-3 w-3 rounded-full" style={{ background: "var(--cf-accent)" }} />
          <span className="flex items-end gap-[3px]">
            {[0.5, 0.85, 1, 0.7, 0.45].map((h, i) => (
              <span
                key={i}
                className="w-[3px] rounded-full"
                style={{
                  height: `${4 + h * 12}px`,
                  background: "var(--cf-listen)",
                  animation: `processing-wave 1.2s ease-in-out ${i * 120}ms infinite`,
                }}
              />
            ))}
          </span>
        </div>
        {/* accent → accent2 sweep to read the gradient identity */}
        <div
          className="absolute inset-x-0 top-0 h-[3px]"
          style={{ background: "linear-gradient(90deg, var(--cf-accent), var(--cf-accent-2))" }}
        />
      </div>

      {/* Caption */}
      <div className="w-full px-3 py-2.5" style={{ background: "var(--cf-bg-elevated)" }}>
        <div className="flex items-center justify-between gap-2">
          <span className="truncate font-display text-[13px] font-semibold" style={{ color: "var(--cf-text)" }}>
            {theme.name}
          </span>
          {active && (
            <span
              className="shrink-0 rounded-full px-1.5 py-px text-[9px] font-bold uppercase tracking-wider"
              style={{ background: "var(--cf-accent)", color: "var(--cf-accent-contrast)" }}
            >
              Active
            </span>
          )}
        </div>
        <p className="mt-0.5 truncate text-[11.5px]" style={{ color: "var(--cf-text-muted)" }}>
          {theme.blurb}
        </p>
      </div>
    </button>
  );
}
