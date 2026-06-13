/**
 * Theme contract for ContextFlow.
 *
 * A theme is a *visual skin only*. Per the product constraint, themes must
 * never change layout, structure, or behaviour — they touch colour and motion
 * exclusively. Both UI surfaces (the floating bubble and the settings window)
 * read the same semantic tokens, so a single theme re-skins everything.
 *
 * Authoring a new theme = filling in one `ThemeColors` object + picking a
 * `MotionStyle`. `applyTheme` expands the typed colours into `--cf-*` CSS
 * custom properties on `:root`; nothing else needs to change.
 */

/** Light vs dark — drives `color-scheme` and a few contrast heuristics. */
export type Appearance = "dark" | "light";

/**
 * Motion personality. Maps to a `data-cf-motion` attribute on `<html>` so CSS
 * can swap keyframe flavours (e.g. flame flicker, neon pulse, glitch) without
 * any JS. Every flavour preserves the *same* element structure and timing
 * envelope — only the easing/glow character differs.
 */
export type MotionStyle =
  | "smooth" // the original calm conic spin (default)
  | "crisp" // tighter, lower-blur, slightly faster — for minimal themes
  | "pulse" // neon breathing glow
  | "glitch" // occasional chromatic shift — cyberpunk
  | "flames" // upward flicker — White Flames / Ember
  | "wave" // fluid lateral sway — Oceanic
  | "aurora" // slow drifting gradient — Nord / Aurora
  | "matrix"; // terminal blink — Matrix / Terminal Green

/** Loose grouping used only to organise the Appearance picker grid. */
export type ThemeGroup =
  | "signature"
  | "editor"
  | "neon"
  | "light"
  | "nature"
  | "mono";

/**
 * Semantic colour tokens. Names describe *role*, not hue, so the bubble and
 * settings window can share them. Any CSS-colour string is valid (hex, rgb,
 * rgba, gradients are NOT allowed here — use stops via accent/accent2).
 */
export interface ThemeColors {
  /** Settings-window canvas. */
  bg: string;
  /** Cards, panels, sidebars — one step above the canvas. */
  bgElevated: string;
  /** Inputs, recessed wells — one step below elevated. */
  bgInset: string;

  /** Primary readable text. */
  text: string;
  /** Secondary text — labels, descriptions. */
  textMuted: string;
  /** Tertiary text — hints, placeholders, disabled. */
  textFaint: string;

  /** Hairline dividers and control outlines. */
  border: string;
  /** Emphasised borders — focus rings, active controls. */
  borderStrong: string;

  /** Brand accent — primary actions, active nav, focus. */
  accent: string;
  /** Accent partner — the far stop of brand gradients. */
  accent2: string;
  /** Readable text/icon colour when placed on top of `accent`. */
  accentContrast: string;

  /** Listening-state accent (bubble) — near stop. */
  listen: string;
  /** Listening-state accent — far stop. */
  listen2: string;
  /** Processing-state accent — near stop. */
  process: string;
  /** Processing-state accent — far stop. */
  process2: string;
  /** Error-state accent — near stop. */
  error: string;
  /** Error-state accent — far stop. */
  error2: string;

  /** Idle visualizer dot colour. */
  idleDot: string;

  /** Floating-bubble outer backdrop (often translucent). */
  bubbleBg: string;
  /** Floating-bubble inner content box. */
  bubbleSurface: string;

  /** Base glow colour as `r, g, b` triplet (consumed inside rgba()). */
  glowRgb: string;
}

export interface Theme {
  /** Stable kebab-case id persisted in settings. Never rename. */
  id: string;
  /** Human-facing name shown in the picker. */
  name: string;
  /** Short evocative descriptor under the name. */
  blurb: string;
  group: ThemeGroup;
  appearance: Appearance;
  motion: MotionStyle;
  colors: ThemeColors;
}
