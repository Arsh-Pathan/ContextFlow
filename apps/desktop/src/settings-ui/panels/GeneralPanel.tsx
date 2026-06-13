/**
 * General panel — at-a-glance status and a couple of always-safe controls.
 * The push-to-talk hotkey is shown read-only here (it is owned by the Rust
 * shell in Slice 1); a future slice makes it editable.
 */
import { AI_PROVIDER_BY_ID, getThemeName, useSettings } from "../_summary";
import { Row, Section } from "../components/Field";

export function GeneralPanel() {
  const { settings, resetAll } = useSettings();
  const provider = AI_PROVIDER_BY_ID[settings.ai.provider];

  return (
    <div className="cf-rise">
      <header className="mb-6">
        <h2 className="font-display text-[22px] font-semibold text-cf-text">
          General
        </h2>
        <p className="mt-1 text-[14px] text-cf-muted">
          A snapshot of how ContextFlow is configured right now.
        </p>
      </header>

      <Section title="Status">
        <Row
          label="Push-to-talk"
          hint="Hold to dictate. Configurable in a later release."
          control={
            <kbd className="rounded-md border border-cf-border-strong bg-cf-inset px-2.5 py-1 font-mono text-[12.5px] text-cf-text">
              Ctrl + Space
            </kbd>
          }
        />
        <Row
          label="Active theme"
          control={
            <span className="text-[13.5px] font-medium text-cf-text">
              {getThemeName(settings.themeId)}
            </span>
          }
        />
        <Row
          label="AI provider"
          control={
            <span className="text-[13.5px] font-medium text-cf-text">
              {provider.name}
              <span className="ml-2 text-cf-faint">
                {settings.features.aiClarification ? "(active)" : "(idle)"}
              </span>
            </span>
          }
        />
      </Section>

      <Section
        title="Reset"
        description="Restore every setting — theme, provider, and feature flags — to defaults."
      >
        <Row
          label="Reset to defaults"
          hint="This returns the app to its original out-of-the-box behaviour."
          control={
            <button
              type="button"
              onClick={resetAll}
              className="rounded-lg border border-cf-border-strong px-3.5 py-2 text-[13px] font-medium
                text-cf-text transition-colors hover:border-cf-accent hover:text-cf-accent
                focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-cf-accent"
            >
              Reset all
            </button>
          }
        />
      </Section>
    </div>
  );
}
