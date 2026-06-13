/**
 * Settings window root.
 *
 * Layout: a WhisperFlow-style two-pane shell — a custom draggable titlebar, a
 * left sidebar of sections, and a scrolling content pane that swaps panels.
 * The whole surface is themed via `--cf-*`, so switching themes restyles the
 * settings window live alongside the bubble.
 *
 * The window is `transparent: true` + `decorations: false`, so we paint our
 * own rounded, bordered canvas and provide window controls.
 */
import { useState, type ComponentType } from "react";
import {
  Settings as GeneralIcon,
  Palette,
  Sparkles,
  ToggleRight,
  Info,
  Minus,
  X,
} from "lucide-react";
import { getCurrentWindow } from "@tauri-apps/api/window";

import { useSettings } from "../settings";
import { getTheme } from "../theme";
import { GeneralPanel } from "./panels/GeneralPanel";
import { AppearancePanel } from "./panels/AppearancePanel";
import { AiProviderPanel } from "./panels/AiProviderPanel";
import { FeaturesPanel } from "./panels/FeaturesPanel";
import { AboutPanel } from "./panels/AboutPanel";
import { AmbientBackground } from "./components/AmbientBackground";
import { Logo } from "../components/Logo";

type SectionId = "general" | "appearance" | "ai" | "features" | "about";

interface NavItem {
  id: SectionId;
  label: string;
  icon: ComponentType<{ size?: number; strokeWidth?: number }>;
  Panel: ComponentType;
}

const NAV: NavItem[] = [
  { id: "general", label: "General", icon: GeneralIcon, Panel: GeneralPanel },
  { id: "appearance", label: "Appearance", icon: Palette, Panel: AppearancePanel },
  { id: "ai", label: "AI Provider", icon: Sparkles, Panel: AiProviderPanel },
  { id: "features", label: "Features", icon: ToggleRight, Panel: FeaturesPanel },
  { id: "about", label: "About", icon: Info, Panel: AboutPanel },
];

export function SettingsApp() {
  const [section, setSection] = useState<SectionId>("general");
  const { settings } = useSettings();
  const theme = getTheme(settings.themeId);

  const active = NAV.find((n) => n.id === section) ?? NAV[0]!;
  const ActivePanel = active.Panel;

  const win = getCurrentWindow();
  const startDrag = () => win.startDragging().catch(() => {});
  const minimize = () => win.minimize().catch(() => {});
  const hide = () => win.hide().catch(() => {});

  return (
    <div
      className="relative flex h-screen w-screen overflow-hidden rounded-[14px] border border-cf-border bg-cf-bg text-cf-text"
      style={{ boxShadow: "0 24px 80px rgba(0,0,0,0.55)" }}
    >
      {settings.features.ambientBackground && <AmbientBackground />}

      {/* ── Sidebar ─────────────────────────────────────────────── */}
      <aside className="relative z-10 flex w-[216px] shrink-0 flex-col border-r border-cf-border bg-cf-elevated/70">
        {/* Brand / drag handle */}
        <div
          className="flex items-center gap-2.5 px-4 pb-3 pt-4"
          onMouseDown={startDrag}
          data-tauri-drag-region
        >
          <div
            className="flex h-8 w-8 items-center justify-center rounded-lg"
            style={{
              background: "var(--cf-bubble-surface)",
              boxShadow: "0 0 14px rgba(var(--cf-glow-rgb),0.45)",
            }}
          >
            <Logo className="h-5 w-5 object-contain" />
          </div>
          <div className="leading-tight">
            <div className="font-display text-[14px] font-semibold text-cf-text">
              ContextFlow
            </div>
            <div className="font-mono text-[10px] text-cf-faint">settings</div>
          </div>
        </div>

        <nav className="flex-1 space-y-0.5 px-2.5 py-2">
          {NAV.map((item) => {
            const Icon = item.icon;
            const isActive = item.id === section;
            return (
              <button
                key={item.id}
                type="button"
                onClick={() => setSection(item.id)}
                className={`group flex w-full items-center gap-3 rounded-lg px-3 py-2 text-[13.5px] font-medium
                  transition-all duration-200 outline-none focus-visible:ring-2 focus-visible:ring-cf-accent
                  ${isActive
                    ? "text-cf-accent-contrast"
                    : "text-cf-muted hover:bg-cf-inset hover:text-cf-text"}`}
                style={
                  isActive
                    ? {
                        background: "var(--cf-accent)",
                        boxShadow: "0 4px 14px rgba(var(--cf-glow-rgb),0.35)",
                      }
                    : undefined
                }
              >
                <Icon size={16} strokeWidth={2} />
                {item.label}
              </button>
            );
          })}
        </nav>

        <div className="px-4 py-3 font-mono text-[10px] text-cf-faint">
          {theme.name}
        </div>
      </aside>

      {/* ── Content ─────────────────────────────────────────────── */}
      <main className="relative z-10 flex min-w-0 flex-1 flex-col">
        {/* Titlebar */}
        <div
          className="flex h-11 shrink-0 items-center justify-end gap-1 px-2"
          onMouseDown={startDrag}
          data-tauri-drag-region
        >
          <button
            type="button"
            onClick={minimize}
            aria-label="Minimize"
            className="flex h-8 w-8 items-center justify-center rounded-md text-cf-muted
              transition-colors hover:bg-cf-inset hover:text-cf-text"
          >
            <Minus size={15} strokeWidth={2.5} />
          </button>
          <button
            type="button"
            onClick={hide}
            aria-label="Close"
            className="flex h-8 w-8 items-center justify-center rounded-md text-cf-muted
              transition-colors hover:bg-rose-500/90 hover:text-white"
          >
            <X size={15} strokeWidth={2.5} />
          </button>
        </div>

        {/* Scrolling panel */}
        <div className="cf-scroll min-h-0 flex-1 overflow-y-auto px-8 pb-10 pt-1">
          <div className="mx-auto max-w-[680px]">
            <ActivePanel />
          </div>
        </div>
      </main>
    </div>
  );
}
