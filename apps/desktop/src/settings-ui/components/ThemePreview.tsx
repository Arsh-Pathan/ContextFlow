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
import { Logo } from "../../components/Logo";

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
        {/* faithful mini bubble recreation (listening state) */}
        <div className="absolute bottom-1/2 translate-y-1/2 left-1/2 flex -translate-x-1/2 items-center justify-center pointer-events-none">
          <div
            className="relative h-[34px] w-[136px] rounded-[17px] overflow-hidden p-[1.5px]"
            style={{
              boxShadow: "0 0 16px color-mix(in srgb, var(--cf-listen) 50%, transparent)",
              ["--cf-aurora-gradient" as string]: "conic-gradient(from 0deg, transparent 0%, var(--cf-listen-2) 30%, var(--cf-listen) 50%, var(--cf-listen-2) 70%, transparent 100%)"
            }}
            data-status="listening"
          >
            {/* aurora gradient background */}
            <div
              className="cf-aurora absolute -inset-[150%] rounded-full opacity-90"
              style={{
                background: "var(--cf-aurora-gradient)",
                filter: "blur(4px)",
                animation: "spin-gradient 2s linear infinite"
              }}
            />
            {/* motion overlays */}
            <div className="cf-flame-overlay" aria-hidden />
            <div className="cf-glitch-overlay" aria-hidden />

            {/* inner bubble box */}
            <div className="relative w-full h-full rounded-[15.5px] flex items-center" style={{ background: "var(--cf-bubble-surface)" }}>
              {/* Logo icon */}
              <div className="absolute left-2 top-1/2 -translate-y-1/2 flex items-center justify-center w-5 h-5 rounded-full overflow-hidden">
                <Logo
                  className="w-full h-full object-cover brightness-110"
                  style={{
                    ["--logo-color-1" as string]: "var(--cf-accent)",
                    ["--logo-color-2" as string]: "var(--cf-accent-2)"
                  }}
                />
              </div>

              {/* Visualizer dots */}
              <div className="absolute left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 flex gap-[4px] items-center justify-center h-6">
                {[0.3, 0.6, 1, 0.7, 0.4, 0.2].map((h, i) => (
                  <span
                    key={i}
                    className="w-[4px] rounded-full origin-center"
                    style={{
                      height: `${4 + h * 12}px`,
                      background: "var(--cf-text)",
                      animation: `processing-wave 1.2s ease-in-out ${i * 120}ms infinite`,
                    }}
                  />
                ))}
              </div>

              {/* Fake close button */}
              <div className="absolute right-2 top-1/2 -translate-y-1/2 flex items-center justify-center w-4 h-4 rounded-full bg-white/5 text-gray-400">
                <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round"><path d="M18 6 6 18"/><path d="m6 6 12 12"/></svg>
              </div>
            </div>
          </div>
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
