/**
 * About panel — identity, philosophy, and a live themed flourish.
 */
import { THEMES } from "../../theme";
import { Logo } from "../../components/Logo";

export function AboutPanel() {
  return (
    <div className="cf-rise">
      <header className="mb-6">
        <h2 className="font-display text-[22px] font-semibold text-cf-text">
          About
        </h2>
      </header>

      <div className="overflow-hidden rounded-2xl border border-cf-border bg-cf-elevated">
        {/* Themed banner */}
        <div
          className="relative flex h-32 items-center gap-4 px-7"
          style={{
            background:
              "radial-gradient(120% 140% at 0% 0%, rgba(var(--cf-glow-rgb),0.22), transparent 60%), var(--cf-bg-inset)",
          }}
        >
          <div
            className="flex h-16 w-16 items-center justify-center rounded-2xl"
            style={{
              background: "var(--cf-bubble-surface)",
              boxShadow: "0 0 28px rgba(var(--cf-glow-rgb),0.5)",
            }}
          >
            <Logo
              className="h-10 w-10 object-contain"
            />
          </div>
          <div>
            <h3 className="font-display text-[20px] font-semibold text-cf-text">
              ContextFlow
            </h3>
            <p className="font-mono text-[12px] text-cf-muted">v0.1.0 · Windows</p>
          </div>
        </div>

        <div className="space-y-4 px-7 py-6">
          <p className="text-[14px] leading-relaxed text-cf-muted">
            Windows-native AI voice dictation — press a hotkey, speak naturally,
            and have polished text appear in any application. Fully offline by
            default, sub-300&nbsp;ms latency, everywhere on Windows.
          </p>
          <div className="grid grid-cols-2 gap-3 sm:grid-cols-3">
            {[
              ["Themes", String(THEMES.length)],
              ["Speech", "Whisper · Windows SR"],
              ["Privacy", "On-device default"],
            ].map(([k, v]) => (
              <div
                key={k}
                className="rounded-xl border border-cf-border bg-cf-inset px-4 py-3"
              >
                <div className="text-[11px] font-semibold uppercase tracking-wider text-cf-faint">
                  {k}
                </div>
                <div className="mt-1 text-[14px] font-medium text-cf-text">{v}</div>
              </div>
            ))}
          </div>
        </div>
      </div>

      <p className="mt-4 text-center text-[12px] text-cf-faint">
        Your thoughts, in flow.
      </p>
    </div>
  );
}
