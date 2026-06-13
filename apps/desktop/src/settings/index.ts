/** Public surface of the settings module. */
export type {
  Settings,
  AiConfig,
  AiProviderId,
  AiProviderMeta,
  FeatureFlags,
  FeatureFlagMeta,
} from "./schema";
export {
  DEFAULT_SETTINGS,
  AI_PROVIDERS,
  AI_PROVIDER_BY_ID,
  FEATURE_FLAGS,
  DEFAULT_FEATURE_FLAGS,
  SETTINGS_VERSION,
} from "./schema";
export { SettingsProvider, useSettings } from "./context";
export {
  loadSettings,
  saveSettings,
  subscribeSettings,
  EVENT_SETTINGS_CHANGED,
} from "./store";
