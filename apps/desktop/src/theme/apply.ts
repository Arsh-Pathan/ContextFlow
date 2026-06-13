/**
 * Expands a `Theme` into `--cf-*` CSS custom properties on a target element
 * (default `:root`) and sets the motion + appearance attributes.
 *
 * This is the *only* place that bridges the typed theme objects to the DOM.
 * Both windows call `applyTheme` on mount and whenever the active theme
 * changes, so the bubble and the settings window always stay in lockstep.
 */

import type { Theme } from "./types";

/** Maps each `ThemeColors` key to its CSS custom-property name. */
const VAR_MAP: Record<keyof Theme["colors"], string> = {
  bg: "--cf-bg",
  bgElevated: "--cf-bg-elevated",
  bgInset: "--cf-bg-inset",
  text: "--cf-text",
  textMuted: "--cf-text-muted",
  textFaint: "--cf-text-faint",
  border: "--cf-border",
  borderStrong: "--cf-border-strong",
  accent: "--cf-accent",
  accent2: "--cf-accent-2",
  accentContrast: "--cf-accent-contrast",
  listen: "--cf-listen",
  listen2: "--cf-listen-2",
  process: "--cf-process",
  process2: "--cf-process-2",
  error: "--cf-error",
  error2: "--cf-error-2",
  idleDot: "--cf-idle-dot",
  bubbleBg: "--cf-bubble-bg",
  bubbleSurface: "--cf-bubble-surface",
  glowRgb: "--cf-glow-rgb",
};

/**
 * Apply `theme` to `target`'s inline custom properties.
 *
 * @param theme  the theme to apply
 * @param target element to set variables on; defaults to `documentElement`.
 *               Pass a preview node to scope a theme to a swatch.
 */
export function applyTheme(
  theme: Theme,
  target: HTMLElement = document.documentElement,
): void {
  const { colors } = theme;
  for (const key of Object.keys(VAR_MAP) as (keyof Theme["colors"])[]) {
    target.style.setProperty(VAR_MAP[key], colors[key]);
  }

  // Attributes drive CSS-only motion + light/dark heuristics. Only touch the
  // documentElement for these — a scoped preview keeps the host's motion.
  if (target === document.documentElement) {
    target.setAttribute("data-cf-motion", theme.motion);
    target.setAttribute("data-cf-appearance", theme.appearance);
    target.style.colorScheme = theme.appearance;
  }
}

/**
 * Build an inline-style object for the same variables, for scoping a theme to
 * a React element (used by the Appearance preview swatches). Returns a plain
 * object spreadable into `style={...}`.
 */
export function themeStyleVars(theme: Theme): Record<string, string> {
  const out: Record<string, string> = {};
  const { colors } = theme;
  for (const key of Object.keys(VAR_MAP) as (keyof Theme["colors"])[]) {
    out[VAR_MAP[key]] = colors[key];
  }
  return out;
}
