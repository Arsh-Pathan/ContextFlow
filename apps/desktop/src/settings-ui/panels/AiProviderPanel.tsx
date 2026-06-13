/**
 * AI Provider panel.
 *
 * Lets the user choose which service powers clarification/cleanup. The
 * Built-in local pipeline is the DEFAULT and needs no key or network — picking
 * any other provider is purely opt-in and only takes effect once the
 * `aiClarification` feature flag is enabled (surfaced here with a live notice).
 *
 * API keys are NEVER stored here — the field explains they go to the OS
 * credential manager via the Rust layer. (Wiring that store is a backend task;
 * this panel captures intent and provider/model selection.)
 */
import {
  AI_PROVIDERS,
  AI_PROVIDER_BY_ID,
  useSettings,
  type AiProviderId,
} from "../../settings";
import { Row, Section, Select, TextInput } from "../components/Field";
import { Toggle } from "../components/Toggle";

const CLEANUP_OPTIONS = [
  { value: "off", label: "Off — verbatim transcript" },
  { value: "light", label: "Light — punctuation only" },
  { value: "standard", label: "Standard — fillers + grammar (default)" },
  { value: "aggressive", label: "Aggressive — rewrite for clarity" },
];

export function AiProviderPanel() {
  const { settings, setAi, setFeature } = useSettings();
  const { ai, features } = settings;
  const meta = AI_PROVIDER_BY_ID[ai.provider];

  const choose = (id: AiProviderId) => {
    const next = AI_PROVIDER_BY_ID[id];
    // Reset model + baseUrl to the new provider's defaults when switching.
    setAi({
      provider: id,
      model: next.models[0] ?? "",
      baseUrl: next.defaultBaseUrl ?? "",
    });
  };

  return (
    <div className="cf-rise">
      <header className="mb-6">
        <h2 className="font-display text-[22px] font-semibold text-cf-text">
          AI Provider
        </h2>
        <p className="mt-1 text-[14px] text-cf-muted">
          Choose what cleans up your dictation. The Built-in pipeline runs fully
          on-device and is the default; cloud and local-LLM options are opt-in.
        </p>
      </header>

      {/* Provider cards */}
      <div className="mb-7 grid grid-cols-1 gap-3 sm:grid-cols-2">
        {AI_PROVIDERS.map((p) => {
          const active = ai.provider === p.id;
          return (
            <button
              key={p.id}
              type="button"
              onClick={() => choose(p.id)}
              aria-pressed={active}
              className={`relative flex flex-col rounded-xl border p-4 text-left transition-all duration-200
                outline-none focus-visible:ring-2 focus-visible:ring-cf-accent
                ${active
                  ? "border-cf-accent bg-cf-elevated shadow-[0_0_0_1px_var(--cf-accent),0_6px_22px_rgba(var(--cf-glow-rgb),0.20)]"
                  : "border-cf-border bg-cf-elevated/60 hover:border-cf-border-strong"}`}
            >
              <div className="mb-1.5 flex items-center justify-between gap-2">
                <span className="font-display text-[15px] font-semibold text-cf-text">
                  {p.name}
                </span>
                <span
                  className={`shrink-0 rounded-full border px-2 py-0.5 text-[10px] font-semibold uppercase tracking-wide
                    ${p.cloud
                      ? "border-amber-500/40 text-amber-400"
                      : "border-emerald-500/40 text-emerald-400"}`}
                >
                  {p.cloud ? "Cloud" : "On-device"}
                </span>
              </div>
              <p className="text-[12.5px] leading-relaxed text-cf-muted">{p.blurb}</p>
              {active && (
                <span className="absolute right-3 top-3 h-2 w-2 rounded-full bg-cf-accent shadow-[0_0_8px_var(--cf-accent)]" />
              )}
            </button>
          );
        })}
      </div>

      {/* Per-provider configuration */}
      <Section
        title={`${meta.name} configuration`}
        description={
          ai.provider === "builtin"
            ? "The built-in pipeline has no configurable model or endpoint."
            : "Stored locally. Keys are kept in the Windows Credential Manager, never in plain settings."
        }
      >
        <Row
          label="Cleanup level"
          hint="How aggressively transcripts are polished."
          control={
            <Select
              value={ai.cleanupLevel}
              onChange={(v) =>
                setAi({ cleanupLevel: v as typeof ai.cleanupLevel })
              }
              options={CLEANUP_OPTIONS}
            />
          }
        />

        {meta.models.length > 1 && (
          <Row
            label="Model"
            hint="Which model to call for cleanup and commands."
            control={
              <Select
                value={ai.model}
                onChange={(v) => setAi({ model: v })}
                options={meta.models.map((m) => ({ value: m, label: m }))}
              />
            }
          />
        )}

        {meta.needsApiKey && (
          <Row
            label="API key"
            hint="Saved to the OS credential vault on apply — not written to disk in plain text."
            control={
              <TextInput
                type="password"
                value=""
                onChange={() => {
                  /* key persistence is handled by the backend credential store */
                }}
                placeholder={`Paste ${meta.name} API key…`}
                mono
              />
            }
          />
        )}

        {meta.hasBaseUrl && (
          <Row
            label="Base URL"
            hint="Override for self-hosted or compatible endpoints."
            control={
              <TextInput
                value={ai.baseUrl}
                onChange={(v) => setAi({ baseUrl: v })}
                placeholder={meta.defaultBaseUrl ?? "https://…"}
                mono
              />
            }
          />
        )}
      </Section>

      {/* Enablement notice — make the dependency on the flag explicit. */}
      <Section
        title="Activation"
        description="AI clarification is a feature flag, off by default, so the app stays verbatim until you opt in."
      >
        <Row
          label="Enable AI clarification"
          hint={
            features.aiClarification
              ? `Active — final transcripts are sent through ${meta.name}.`
              : "Off — transcripts are inserted exactly as recognised."
          }
          htmlFor="ai-enable"
          control={
            <Toggle
              id="ai-enable"
              checked={features.aiClarification}
              onChange={(v) => setFeature("aiClarification", v)}
              label="Enable AI clarification"
            />
          }
        />
      </Section>
    </div>
  );
}
