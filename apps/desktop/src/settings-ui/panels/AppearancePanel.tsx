/**
 * Appearance panel: the live theme gallery. Themes are grouped; each swatch is
 * a scoped live preview. Selecting one applies it instantly to *both* windows
 * (via the settings store broadcast).
 */
import { useMemo } from "react";

import { THEMES, type ThemeGroup } from "../../theme";
import { useSettings } from "../../settings";
import { Row, Section } from "../components/Field";
import { Toggle as Switch } from "../components/Toggle";
import { ThemePreview } from "../components/ThemePreview";

const GROUP_LABEL: Record<ThemeGroup, string> = {
  signature: "Signature",
  editor: "Editor Classics",
  neon: "Neon & Cyber",
  nature: "Nature",
  mono: "Monochrome",
  light: "Light",
};

const GROUP_ORDER: ThemeGroup[] = [
  "signature",
  "editor",
  "neon",
  "nature",
  "mono",
  "light",
];

export function AppearancePanel() {
  const { settings, setTheme, setReduceMotion } = useSettings();

  const grouped = useMemo(() => {
    const map = new Map<ThemeGroup, typeof THEMES>();
    for (const g of GROUP_ORDER) map.set(g, []);
    for (const t of THEMES) map.get(t.group)?.push(t);
    return map;
  }, []);

  return (
    <div className="cf-rise">
      <header className="mb-6">
        <h2 className="font-display text-[22px] font-semibold text-cf-text">
          Appearance
        </h2>
        <p className="mt-1 text-[14px] text-cf-muted">
          {THEMES.length} themes — colour and motion only. Your layout never
          changes. Selection applies live to the bubble and this window.
        </p>
      </header>

      {GROUP_ORDER.map((group) => {
        const themes = grouped.get(group) ?? [];
        if (themes.length === 0) return null;
        return (
          <div key={group} className="mb-7">
            <h3 className="mb-3 font-display text-[12px] font-semibold uppercase tracking-[0.16em] text-cf-faint">
              {GROUP_LABEL[group]}
            </h3>
            <div className="grid grid-cols-2 gap-3 sm:grid-cols-3 xl:grid-cols-4">
              {themes.map((theme) => (
                <ThemePreview
                  key={theme.id}
                  theme={theme}
                  active={settings.themeId === theme.id}
                  onSelect={() => setTheme(theme.id)}
                />
              ))}
            </div>
          </div>
        );
      })}

      <Section
        title="Motion"
        description="Decorative animation can be reduced independently of your OS setting."
      >
        <Row
          label="Reduce motion"
          hint="Freeze decorative flourishes (aurora spin, flicker, glitch). Status colours still change."
          htmlFor="reduce-motion"
          control={
            <Switch
              id="reduce-motion"
              checked={settings.reduceMotion}
              onChange={setReduceMotion}
              label="Reduce motion"
            />
          }
        />
      </Section>
    </div>
  );
}
