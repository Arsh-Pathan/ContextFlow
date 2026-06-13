/**
 * ContextFlow UI settings schema.
 *
 * This is the *front-end* settings model: theme, AI-provider choice, and
 * feature flags. It persists to `localStorage` and broadcasts changes across
 * windows via a Tauri event, so the bubble and the settings window stay in
 * sync without any Rust round-trip. (The Rust `core/settings` crate remains
 * the source of truth for engine config like the active speech provider; this
 * layer governs presentation + opt-in UI features only.)
 *
 * GOLDEN RULE: every default here reproduces the app's *current* behaviour.
 * New features are flags that default to `false`. Changing a default is a
 * behaviour change and must be called out in the changelog.
 */

import { DEFAULT_THEME_ID } from "../theme";

/* ─── AI providers ──────────────────────────────────────────────────────────
 * The clarification/cleanup layer is pluggable. The built-in local pipeline is
 * the DEFAULT and requires no key or network. Cloud/local-LLM providers are
 * opt-in; selecting one does not change behaviour until the feature flag
 * `aiClarification` is also enabled (see FEATURE_FLAGS).
 */

export type AiProviderId =
  | "builtin" // current local clarification pipeline (default)
  | "openai"
  | "anthropic"
  | "gemini"
  | "ollama";

export interface AiProviderMeta {
  id: AiProviderId;
  name: string;
  /** One-line description shown in the picker. */
  blurb: string;
  /** Whether the provider sends text off-device. Drives the privacy badge. */
  cloud: boolean;
  /** Whether an API key is required (stored via OS credential manager, not here). */
  needsApiKey: boolean;
  /** Whether a custom base URL is configurable (e.g. Ollama, OpenAI-compatible). */
  hasBaseUrl: boolean;
  /** Suggested model identifiers for the dropdown. First is the default. */
  models: string[];
  /** Default base URL, when applicable. */
  defaultBaseUrl?: string;
}

export const AI_PROVIDERS: AiProviderMeta[] = [
  {
    id: "builtin",
    name: "Built-in (Local)",
    blurb:
      "ContextFlow's on-device clarification — punctuation, filler removal, spoken corrections. No network, no key.",
    cloud: false,
    needsApiKey: false,
    hasBaseUrl: false,
    models: ["contextflow-rules"],
  },
  {
    id: "openai",
    name: "OpenAI",
    blurb: "GPT models via the OpenAI API. Highest polish; text leaves the device.",
    cloud: true,
    needsApiKey: true,
    hasBaseUrl: true,
    models: ["gpt-4o-mini", "gpt-4o", "gpt-4.1-mini", "gpt-4.1"],
    defaultBaseUrl: "https://api.openai.com/v1",
  },
  {
    id: "anthropic",
    name: "Anthropic",
    blurb: "Claude models. Strong instruction-following for tone and rewrites.",
    cloud: true,
    needsApiKey: true,
    hasBaseUrl: false,
    models: [
      "claude-haiku-4-5-20251001",
      "claude-sonnet-4-6",
      "claude-opus-4-8",
    ],
  },
  {
    id: "gemini",
    name: "Google Gemini",
    blurb: "Gemini models via Google AI. Fast, multilingual.",
    cloud: true,
    needsApiKey: true,
    hasBaseUrl: false,
    models: ["gemini-2.0-flash", "gemini-2.5-flash", "gemini-2.5-pro"],
  },
  {
    id: "ollama",
    name: "Ollama (Local LLM)",
    blurb:
      "Run an open model locally via Ollama. Private, offline, your hardware.",
    cloud: false,
    needsApiKey: false,
    hasBaseUrl: true,
    models: ["llama3.2", "qwen2.5", "mistral", "phi4"],
    defaultBaseUrl: "http://localhost:11434",
  },
];

export const AI_PROVIDER_BY_ID: Record<AiProviderId, AiProviderMeta> =
  Object.fromEntries(AI_PROVIDERS.map((p) => [p.id, p])) as Record<
    AiProviderId,
    AiProviderMeta
  >;

export interface AiConfig {
  /** Active provider. `builtin` is the default and current behaviour. */
  provider: AiProviderId;
  /** Selected model id for the active provider. */
  model: string;
  /** Optional override base URL (Ollama / OpenAI-compatible endpoints). */
  baseUrl: string;
  /**
   * Cleanup intensity for whichever provider is active.
   * `standard` mirrors today's behaviour.
   */
  cleanupLevel: "off" | "light" | "standard" | "aggressive";
}

/* ─── Feature flags ─────────────────────────────────────────────────────────
 * Every flag is OFF by default. Turning them all off === current app. Each
 * entry is self-describing so the Features panel renders from this registry.
 */

export interface FeatureFlagMeta {
  key: keyof FeatureFlags;
  name: string;
  blurb: string;
  /** Optional grouping label in the Features panel. */
  category: "Intelligence" | "Interface" | "Workflow";
  /** Marks experimental flags with a badge. */
  experimental?: boolean;
}

export interface FeatureFlags {
  /** Route final transcripts through the selected AI provider for cleanup. */
  aiClarification: boolean;
  /** Show live partial transcripts inside the bubble while speaking. */
  livePreview: boolean;
  /** Voice commands ("make professional", "bullet points", …). */
  voiceCommands: boolean;
  /** Personal dictionary that learns from corrections. */
  personalDictionary: boolean;
  /** Snippets / voice macros with variable substitution. */
  snippets: boolean;
  /** Per-app profiles (code / email / chat) tune cleanup + casing. */
  perAppProfiles: boolean;
  /** Subtle sound cues on start/stop/insert. */
  soundCues: boolean;
  /** Keep a local history of recent dictations (never leaves the device). */
  dictationHistory: boolean;
  /** Animated theme-aware background in the settings window. */
  ambientBackground: boolean;
}

export const DEFAULT_FEATURE_FLAGS: FeatureFlags = {
  aiClarification: false,
  livePreview: false,
  voiceCommands: false,
  personalDictionary: false,
  snippets: false,
  perAppProfiles: false,
  soundCues: false,
  dictationHistory: false,
  ambientBackground: false,
};

export const FEATURE_FLAGS: FeatureFlagMeta[] = [
  {
    key: "aiClarification",
    name: "AI Clarification",
    blurb:
      "Send final transcripts to the selected AI provider for grammar, tone, and rewrites. Uses the Built-in local pipeline unless you pick another provider.",
    category: "Intelligence",
  },
  {
    key: "voiceCommands",
    name: "Voice Commands",
    blurb:
      'Speak transformations like "make this professional" or "turn into bullet points" to reshape selected text.',
    category: "Intelligence",
    experimental: true,
  },
  {
    key: "personalDictionary",
    name: "Personal Dictionary",
    blurb:
      "Teach ContextFlow names, jargon, and spellings. Learns from your corrections over time.",
    category: "Intelligence",
  },
  {
    key: "livePreview",
    name: "Live Transcript Preview",
    blurb: "Show partial results inside the bubble as you speak, before you release.",
    category: "Interface",
    experimental: true,
  },
  {
    key: "ambientBackground",
    name: "Ambient Background",
    blurb: "A slow, theme-aware aurora behind the settings window.",
    category: "Interface",
  },
  {
    key: "soundCues",
    name: "Sound Cues",
    blurb: "Discreet audio feedback when dictation starts, stops, and inserts.",
    category: "Interface",
  },
  {
    key: "snippets",
    name: "Snippets & Macros",
    blurb:
      'Expand spoken triggers into longer text with variables — e.g. "my address" or "standup update".',
    category: "Workflow",
  },
  {
    key: "perAppProfiles",
    name: "Per-App Profiles",
    blurb:
      "Automatically adjust casing and cleanup depending on the focused app (code editor vs. email vs. chat).",
    category: "Workflow",
    experimental: true,
  },
  {
    key: "dictationHistory",
    name: "Dictation History",
    blurb:
      "Keep a searchable local log of recent dictations. Stored on-device only; never uploaded.",
    category: "Workflow",
  },
];

/* ─── Root settings object ──────────────────────────────────────────────────*/

export interface Settings {
  /** Schema version for forward-compatible migrations. */
  version: number;
  /** Active theme id (see src/theme). */
  themeId: string;
  /** Honour OS reduced-motion even for non-decorative transitions. */
  reduceMotion: boolean;
  ai: AiConfig;
  features: FeatureFlags;
}

export const SETTINGS_VERSION = 1;

export const DEFAULT_SETTINGS: Settings = {
  version: SETTINGS_VERSION,
  themeId: DEFAULT_THEME_ID,
  reduceMotion: false,
  ai: {
    provider: "builtin",
    model: "contextflow-rules",
    baseUrl: "",
    cleanupLevel: "standard",
  },
  features: { ...DEFAULT_FEATURE_FLAGS },
};
