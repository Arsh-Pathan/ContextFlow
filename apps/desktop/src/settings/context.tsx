/**
 * React bindings for settings.
 *
 * `SettingsProvider` owns the single source of truth for the current window,
 * applies the active theme to the DOM whenever it changes, and keeps in sync
 * with other windows via the cross-window event. Consumers use the `useSettings`
 * hook to read settings and call typed mutators.
 *
 * Mutators are intentionally granular (setTheme, setAi, setFeature) so call
 * sites read clearly and we never persist a malformed partial.
 */

import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useRef,
  useState,
  type ReactNode,
} from "react";

import { applyTheme, getTheme } from "../theme";
import {
  DEFAULT_SETTINGS,
  type AiConfig,
  type FeatureFlags,
  type Settings,
} from "./schema";
import { loadSettings, saveSettings, subscribeSettings } from "./store";

interface SettingsContextValue {
  settings: Settings;
  /** Switch the active theme by id. Applies immediately + persists. */
  setTheme: (themeId: string) => void;
  /** Patch the AI provider config. */
  setAi: (patch: Partial<AiConfig>) => void;
  /** Toggle or set a single feature flag. */
  setFeature: (key: keyof FeatureFlags, value: boolean) => void;
  /** Toggle reduced-motion preference. */
  setReduceMotion: (value: boolean) => void;
  /** Reset everything to defaults (current app behaviour). */
  resetAll: () => void;
}

const SettingsContext = createContext<SettingsContextValue | null>(null);

export function SettingsProvider({ children }: { children: ReactNode }) {
  const [settings, setSettings] = useState<Settings>(() => loadSettings());

  // Guard so that applying an *incoming* (remote) change doesn't echo back out
  // as another broadcast, which would ping-pong between windows.
  const applyingRemote = useRef(false);

  // Apply the active theme to the DOM whenever the id changes.
  useEffect(() => {
    applyTheme(getTheme(settings.themeId));
  }, [settings.themeId]);

  // Reflect reduced-motion preference as an attribute CSS can key on.
  useEffect(() => {
    document.documentElement.toggleAttribute(
      "data-cf-reduce-motion",
      settings.reduceMotion,
    );
  }, [settings.reduceMotion]);

  // Listen for changes from other windows.
  useEffect(() => {
    let unlisten: (() => void) | null = null;
    let active = true;
    subscribeSettings((incoming) => {
      applyingRemote.current = true;
      setSettings(incoming);
    })
      .then((fn) => {
        if (active) unlisten = fn;
        else fn();
      })
      .catch(() => {
        /* no Tauri in browser preview */
      });
    return () => {
      active = false;
      if (unlisten) unlisten();
    };
  }, []);

  // Persist + broadcast on every local change (but not when echoing a remote one).
  useEffect(() => {
    if (applyingRemote.current) {
      applyingRemote.current = false;
      return;
    }
    saveSettings(settings);
  }, [settings]);

  const setTheme = useCallback((themeId: string) => {
    setSettings((s) => ({ ...s, themeId }));
  }, []);

  const setAi = useCallback((patch: Partial<AiConfig>) => {
    setSettings((s) => ({ ...s, ai: { ...s.ai, ...patch } }));
  }, []);

  const setFeature = useCallback(
    (key: keyof FeatureFlags, value: boolean) => {
      setSettings((s) => ({
        ...s,
        features: { ...s.features, [key]: value },
      }));
    },
    [],
  );

  const setReduceMotion = useCallback((value: boolean) => {
    setSettings((s) => ({ ...s, reduceMotion: value }));
  }, []);

  const resetAll = useCallback(() => {
    setSettings(structuredClone(DEFAULT_SETTINGS));
  }, []);

  const value = useMemo<SettingsContextValue>(
    () => ({ settings, setTheme, setAi, setFeature, setReduceMotion, resetAll }),
    [settings, setTheme, setAi, setFeature, setReduceMotion, resetAll],
  );

  return (
    <SettingsContext.Provider value={value}>
      {children}
    </SettingsContext.Provider>
  );
}

/** Access settings + mutators. Throws if used outside `SettingsProvider`. */
export function useSettings(): SettingsContextValue {
  const ctx = useContext(SettingsContext);
  if (!ctx) {
    throw new Error("useSettings must be used within a SettingsProvider");
  }
  return ctx;
}
