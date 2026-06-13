/**
 * Small shared helpers for the settings panels — kept here so panels don't
 * reach across module boundaries for one-liners.
 */
import { getTheme } from "../theme";

export { AI_PROVIDER_BY_ID, useSettings } from "../settings";

/** Human-facing theme name for a stored id (falls back to the default). */
export function getThemeName(id: string): string {
  return getTheme(id).name;
}
