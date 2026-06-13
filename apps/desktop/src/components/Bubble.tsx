import { useRef, type CSSProperties } from "react";
import { X } from "lucide-react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Logo } from "./Logo";

export type DictationStatus = "idle" | "listening" | "processing" | "error";

interface BubbleProps {
  status: DictationStatus;
  level?: number;
  message?: string;
  provider?: string;
  warning?: string;
}

const STATUS_LABEL: Record<DictationStatus, string> = {
  idle: "Idle",
  listening: "Listening",
  processing: "Processing",
  error: "Error",
};

const PROVIDER_LABEL: Record<string, string> = {
  "whisper-cpp": "Whisper (Local, GPU)",
  "windows-sr": "Windows SR (Built-in)",
};

/**
 * Per-status colour roles, resolved from the active theme's `--cf-*` tokens.
 * `near`/`far` are the conic-gradient stops; `glow` feeds the box-shadow; `dot`
 * is the visualizer bar colour. The default theme reproduces the original
 * teal/sky/rose palette exactly, so existing users see no change.
 */
const STATUS_COLORS: Record<
  DictationStatus,
  { near: string; far: string; dot: string }
> = {
  idle: { near: "var(--cf-idle-dot)", far: "transparent", dot: "var(--cf-idle-dot)" },
  listening: { near: "var(--cf-listen)", far: "var(--cf-listen-2)", dot: "var(--cf-text)" },
  processing: { near: "var(--cf-process)", far: "var(--cf-process-2)", dot: "var(--cf-process)" },
  error: { near: "var(--cf-error)", far: "var(--cf-error-2)", dot: "var(--cf-error)" },
};

export function Bubble({ status, level, message, provider, warning }: BubbleProps) {
  const tooltipLines = [STATUS_LABEL[status]];
  if (provider) {
    tooltipLines.push(`Provider: ${PROVIDER_LABEL[provider] ?? provider}`);
  }
  if (warning) {
    tooltipLines.push(`Warning: ${warning}`);
  }
  if (status === "error" && message) {
    tooltipLines.push(message);
  }
  const tooltip = tooltipLines.join(" | ");

  const ariaLabel =
    status === "error" && message
      ? `ContextFlow: Error — ${message}`
      : `ContextFlow: ${STATUS_LABEL[status]}`;

  // Track whether volume is rising or falling to adjust CSS transition speed
  const prevLevelRef = useRef(0);
  const raw = level ?? 0;
  const isRising = raw > prevLevelRef.current;
  prevLevelRef.current = raw;

  const colors = STATUS_COLORS[status];

  // 6 dots for the visualizer
  const numDots = 6;
  const dots = Array.from({ length: numDots }).map((_, i) => {
    let heightPx = 5;

    if (status === "listening" && level !== undefined) {
      // Shape the waveform so the middle is taller than the edges
      const distanceToCenter = Math.abs((numDots - 1) / 2 - i);
      const curve = 1 - (distanceToCenter / ((numDots - 1) / 2)) * 0.4;

      // Non-linear boost to give MUCH bigger amplitude even for quiet speech
      const boostedLevel = Math.pow(raw, 0.5);

      // Higher base gain
      const baseGain = boostedLevel * 70;

      // Slowly breathing natural variation per dot so max positions dynamically morph over time
      const timePhase = Date.now() / 400;
      const organicVariation = 0.8 + Math.sin(timePhase + i * 2.4) * 0.4;

      // Cap at 32px so it fits nicely in the 44px bubble
      heightPx = 5 + Math.min(32, baseGain * curve * organicVariation);
    }

    const delayMs = i * 150;
    const delay = `${delayMs}ms`;

    let animation = "none";
    if (status === "processing") {
      animation = `processing-wave 1.2s ease-in-out ${delay} infinite`;
    } else if (status === "idle") {
      animation = `idle-breathe 2s ease-in-out ${delayMs * 2}ms infinite`;
    } else if (status === "error") {
      animation = `error-shake 0.4s ease-in-out ${delay} infinite`;
    }

    const dotStyle: CSSProperties = {
      height:
        status === "listening" ? `${heightPx}px` : status === "idle" ? "5px" : "5px",
      background: colors.dot,
      animation,
    };

    const transitionClass =
      status === "listening"
        ? isRising
          ? "transition-[height] duration-100 ease-out"
          : "transition-[height] duration-500 ease-out"
        : "transition-all duration-300 ease-in-out";

    return (
      <div
        key={i}
        className={`w-[5px] rounded-full origin-center ${transitionClass}`}
        style={dotStyle}
      />
    );
  });

  const handleClose = async () => {
    try {
      await getCurrentWindow().hide();
    } catch (err) {
      console.warn("Failed to hide window on close:", err);
    }
  };

  // Conic-gradient aurora ring. Idle uses a single faint stop (and is hidden by
  // the opacity-0 wrapper anyway); active states use the near/far stop pair.
  const gradientStyle =
    status === "idle"
      ? `conic-gradient(from 0deg, transparent 0%, ${colors.near} 50%, transparent 100%)`
      : `conic-gradient(from 0deg, transparent 0%, ${colors.far} 30%, ${colors.near} 50%, ${colors.far} 70%, transparent 100%)`;

  const spinDuration =
    status === "listening"
      ? "2s"
      : status === "processing"
        ? "0.5s"
        : status === "error"
          ? "0s" // Stop spinning on error
          : "4s";

  // Status-tinted glow; idle keeps the original neutral drop shadow.
  const boxShadow =
    status === "idle"
      ? "0 4px 24px rgba(0,0,0,0.6)"
      : `0 0 24px color-mix(in srgb, ${colors.near} 50%, transparent)`;

  const isExpanded = status !== "idle";
  const wrapperClass = isExpanded ? "w-[180px] opacity-100" : "w-[44px] opacity-0";

  // Expose the resolved gradient to descendant motion overlays (flames/glitch)
  // via a custom property so CSS can paint them without re-deriving stops.
  const wrapperStyle = {
    boxShadow,
    ["--cf-aurora-gradient" as string]: gradientStyle,
  } as CSSProperties;

  return (
    <div
      role="status"
      aria-label={ariaLabel}
      title={tooltip}
      className={`relative h-[44px] rounded-[22px] overflow-hidden p-[2px] transition-all duration-400 ease-[cubic-bezier(0.16,1,0.3,1)] ${wrapperClass}`}
      style={wrapperStyle}
      data-status={status}
    >
      {/* Spinning Gradient Border (themed aurora) */}
      <div
        className="cf-aurora absolute -inset-[150%] rounded-full opacity-90"
        style={{
          background: "var(--cf-aurora-gradient)",
          filter: "blur(6px)",
          animation:
            spinDuration === "0s"
              ? "none"
              : `spin-gradient ${spinDuration} linear infinite`,
        }}
      />

      {/* Motion overlays — only styled under their matching data-cf-motion. */}
      <div className="cf-flame-overlay" aria-hidden />
      <div className="cf-glitch-overlay" aria-hidden />

      {/* Inner Content Box */}
      <div
        className="relative w-full h-full rounded-[20px]"
        style={{ background: "var(--cf-bubble-surface)" }}
      >
        {/* Left Icon (Logo) */}
        <div className="absolute left-2.5 top-1/2 -translate-y-1/2 flex items-center justify-center w-7 h-7 rounded-full overflow-hidden transition-all duration-300">
          <Logo
            className={`w-full h-full object-cover transition-all duration-300 ${
              status !== "idle" ? "brightness-110" : ""
            }`}
            style={{
              ["--logo-color-1" as string]: status === "idle" ? "var(--cf-accent)" : colors.near,
              ["--logo-color-2" as string]: status === "idle" ? "var(--cf-accent-2)" : colors.far,
            }}
          />
        </div>

        {/* Middle Audio Visualizer Dots */}
        <div
          className={`absolute left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 flex gap-[5px] items-center justify-center h-6 transition-opacity duration-300 ${
            isExpanded ? "opacity-100" : "opacity-0"
          }`}
        >
          {dots}
        </div>

        {/* Warning indicator */}
        {warning && (
          <div
            className="absolute right-10 top-1/2 -translate-y-1/2 w-3.5 h-3.5 rounded-full bg-amber-500 flex items-center justify-center shrink-0"
            title={warning}
          >
            <span className="text-black text-[10px] font-bold leading-none">!</span>
          </div>
        )}

        {/* Right Icon (Close Button) */}
        <button
          onClick={handleClose}
          className="absolute right-2.5 top-1/2 -translate-y-1/2 flex items-center justify-center w-6 h-6 rounded-full bg-white/5 text-gray-400 hover:text-white hover:bg-white/15 transition-all duration-200"
        >
          <X size={14} strokeWidth={2.5} />
        </button>
      </div>
    </div>
  );
}
