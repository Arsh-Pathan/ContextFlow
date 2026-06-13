/**
 * Features panel — renders entirely from the FEATURE_FLAGS registry, grouped by
 * category. Every flag is off by default; this is the single opt-in surface.
 */
import { useMemo } from "react";

import { FEATURE_FLAGS, useSettings, type FeatureFlagMeta } from "../../settings";
import { Toggle } from "../components/Toggle";

const CATEGORY_ORDER: FeatureFlagMeta["category"][] = [
  "Intelligence",
  "Interface",
  "Workflow",
];

export function FeaturesPanel() {
  const { settings, setFeature } = useSettings();

  const byCategory = useMemo(() => {
    const map = new Map<FeatureFlagMeta["category"], FeatureFlagMeta[]>();
    for (const c of CATEGORY_ORDER) map.set(c, []);
    for (const f of FEATURE_FLAGS) map.get(f.category)?.push(f);
    return map;
  }, []);

  const enabledCount = FEATURE_FLAGS.filter(
    (f) => settings.features[f.key],
  ).length;

  return (
    <div className="cf-rise">
      <header className="mb-6">
        <h2 className="font-display text-[22px] font-semibold text-cf-text">
          Features
        </h2>
        <p className="mt-1 text-[14px] text-cf-muted">
          Opt-in capabilities. Everything here is off by default — the app
          behaves exactly as before until you switch something on.{" "}
          <span className="text-cf-faint">({enabledCount} enabled)</span>
        </p>
      </header>

      {CATEGORY_ORDER.map((cat) => {
        const flags = byCategory.get(cat) ?? [];
        if (flags.length === 0) return null;
        return (
          <div key={cat} className="mb-7">
            <h3 className="mb-3 font-display text-[12px] font-semibold uppercase tracking-[0.16em] text-cf-faint">
              {cat}
            </h3>
            <div className="overflow-hidden rounded-xl border border-cf-border bg-cf-elevated">
              {flags.map((f) => {
                const on = settings.features[f.key];
                return (
                  <div
                    key={f.key}
                    className="flex items-start justify-between gap-6 border-b border-cf-border px-4 py-4 last:border-b-0"
                  >
                    <div className="min-w-0">
                      <div className="flex items-center gap-2">
                        <span className="text-[14px] font-medium text-cf-text">
                          {f.name}
                        </span>
                        {f.experimental && (
                          <span className="rounded border border-cf-border-strong px-1.5 py-px text-[9px] font-bold uppercase tracking-wider text-cf-faint">
                            Experimental
                          </span>
                        )}
                      </div>
                      <p className="mt-1 text-[12.5px] leading-relaxed text-cf-muted">
                        {f.blurb}
                      </p>
                    </div>
                    <div className="shrink-0 pt-0.5">
                      <Toggle
                        checked={on}
                        onChange={(v) => setFeature(f.key, v)}
                        label={f.name}
                      />
                    </div>
                  </div>
                );
              })}
            </div>
          </div>
        );
      })}
    </div>
  );
}
