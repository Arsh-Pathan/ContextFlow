/**
 * Settings persistence + cross-window sync.
 *
 * Storage: `localStorage` (per-user, synchronous, survives restarts). The
 * webview's localStorage is shared across Tauri windows of the same app, but
 * an *open* window won't see another window's write without a nudge — so we
 * also emit a Tauri event carrying the new settings, and listen for it.
 *
 * The module is intentionally framework-agnostic; React bindings live in
 * `context.tsx`.
 */

import { emit, listen, type UnlistenFn } from "@tauri-apps/api/event";

import {
  DEFAULT_SETTINGS,
  SETTINGS_VERSION,
  type Settings,
  type FeatureFlags,
} from "./schema";

const STORAGE_KEY = "contextflow.settings.v1";

/** Tauri event topic broadcast whenever settings change in any window. */
export const EVENT_SETTINGS_CHANGED = "settings://changed";

/**
 * Deep-merge a partial persisted object over defaults so that settings saved
 * by an older build automatically gain any newly-added fields (e.g. a new
 * feature flag) at their default value. Only the keys we know about survive.
 */
function coerce(raw: unknown): Settings {
  if (!raw || typeof raw !== "object") return structuredClone(DEFAULT_SETTINGS);
  const r = raw as Partial<Settings>;
  return {
    version: SETTINGS_VERSION,
    themeId:
      typeof r.themeId === "string" ? r.themeId : DEFAULT_SETTINGS.themeId,
    reduceMotion:
      typeof r.reduceMotion === "boolean"
        ? r.reduceMotion
        : DEFAULT_SETTINGS.reduceMotion,
    ai: {
      provider: r.ai?.provider ?? DEFAULT_SETTINGS.ai.provider,
      model: r.ai?.model ?? DEFAULT_SETTINGS.ai.model,
      baseUrl: r.ai?.baseUrl ?? DEFAULT_SETTINGS.ai.baseUrl,
      cleanupLevel: r.ai?.cleanupLevel ?? DEFAULT_SETTINGS.ai.cleanupLevel,
    },
    // Start from default flags (all off) and overlay any known booleans.
    features: mergeFlags(r.features),
  };
}

function mergeFlags(saved: Partial<FeatureFlags> | undefined): FeatureFlags {
  const out = { ...DEFAULT_SETTINGS.features };
  if (saved && typeof saved === "object") {
    for (const k of Object.keys(out) as (keyof FeatureFlags)[]) {
      if (typeof saved[k] === "boolean") out[k] = saved[k] as boolean;
    }
  }
  return out;
}

/** Read settings synchronously from localStorage, coerced over defaults. */
export function loadSettings(): Settings {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return structuredClone(DEFAULT_SETTINGS);
    return coerce(JSON.parse(raw));
  } catch (err) {
    console.warn("ContextFlow: failed to load settings, using defaults:", err);
    return structuredClone(DEFAULT_SETTINGS);
  }
}

/**
 * Persist settings to localStorage and broadcast to other windows.
 *
 * `emit` is best-effort: when running in a plain browser (no Tauri) it throws,
 * which we swallow — localStorage alone is enough for single-window dev.
 */
export function saveSettings(settings: Settings): void {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(settings));
  } catch (err) {
    console.warn("ContextFlow: failed to persist settings:", err);
  }
  emit(EVENT_SETTINGS_CHANGED, settings).catch(() => {
    /* no Tauri (browser preview) — localStorage suffices */
  });
}

/**
 * Subscribe to settings changes broadcast by *other* windows. Returns a
 * Promise resolving to an unlisten function. Safe to call without Tauri (it
 * just never fires).
 */
export async function subscribeSettings(
  onChange: (settings: Settings) => void,
): Promise<UnlistenFn> {
  return listen<Settings>(EVENT_SETTINGS_CHANGED, (event) => {
    onChange(coerce(event.payload));
  });
}
