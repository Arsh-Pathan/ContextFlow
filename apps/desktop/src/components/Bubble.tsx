import { useRef, type CSSProperties } from "react";
import { X } from "lucide-react";
import { getCurrentWindow } from "@tauri-apps/api/window";

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

    let animation = 'none';
    if (status === "processing") {
      animation = `processing-wave 1.2s ease-in-out ${delay} infinite`;
    } else if (status === "idle") {
      animation = `idle-breathe 2s ease-in-out ${delayMs * 2}ms infinite`;
    } else if (status === "error") {
      animation = `error-shake 0.4s ease-in-out ${delay} infinite`;
    }

    const dotStyle: CSSProperties = {
      height: status === "listening" ? `${heightPx}px` : undefined,
      animation: animation,
    };

    const transitionClass = status === "listening"
      ? (isRising ? "transition-[height] duration-100 ease-out" : "transition-[height] duration-500 ease-out")
      : "transition-all duration-300 ease-in-out";

    return (
      <div
        key={i}
        className={`w-[5px] rounded-full origin-center ${transitionClass} ${
          status === "processing" 
            ? "h-[5px] bg-sky-400" 
            : status === "error"
              ? "h-[5px] bg-rose-500"
              : status === "listening"
                ? "bg-white"
                : "h-[5px] bg-gray-500"
        }`}
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

  let shadowClass = "shadow-[0_4px_24px_rgba(0,0,0,0.6)]";
  let gradientStyle = "conic-gradient(from 0deg, transparent 0%, #333 50%, transparent 100%)";
  let spinDuration = "4s";

  if (status === "listening") {
    shadowClass = "shadow-[0_0_24px_rgba(16,185,129,0.5)]";
    gradientStyle = "conic-gradient(from 0deg, transparent 0%, #0e7490 30%, #10b981 50%, #0e7490 70%, transparent 100%)";
    spinDuration = "2s";
  } else if (status === "processing") {
    shadowClass = "shadow-[0_0_24px_rgba(14,116,144,0.5)]";
    gradientStyle = "conic-gradient(from 0deg, transparent 0%, #0ea5e9 30%, #0e7490 50%, #0ea5e9 70%, transparent 100%)";
    spinDuration = "1s";
  } else if (status === "error") {
    shadowClass = "shadow-[0_0_24px_rgba(244,63,94,0.4)]";
    gradientStyle = "conic-gradient(from 0deg, transparent 0%, #be123c 30%, #f43f5e 50%, #be123c 70%, transparent 100%)";
    spinDuration = "0s"; // Stop spinning on error
  }

  const isExpanded = status !== "idle";
  const wrapperClass = isExpanded ? "w-[180px] opacity-100" : "w-[44px] opacity-0";

  return (
    <div
      role="status"
      aria-label={ariaLabel}
      title={tooltip}
      className={`relative h-[44px] rounded-[22px] overflow-hidden p-[2px] transition-all duration-400 ease-[cubic-bezier(0.16,1,0.3,1)] ${wrapperClass} ${shadowClass}`}
      data-status={status}
    >


      {/* Spinning Gradient Border */}
      <div 
        className="absolute -inset-[150%] rounded-full opacity-90"
        style={{
          background: gradientStyle,
          filter: "blur(6px)",
          animation: spinDuration === "0s" ? "none" : `spin-gradient ${spinDuration} linear infinite`
        }}
      />

      {/* Inner Content Box (Glassmorphism) */}
      <div className="relative w-full h-full rounded-[20px] bg-[#1a1a1a]">
        
        {/* Left Icon (Logo) */}
        <div className={`absolute left-2.5 top-1/2 -translate-y-1/2 flex items-center justify-center w-7 h-7 rounded-full overflow-hidden transition-all duration-300 ${
          status === "listening" ? "drop-shadow-[0_0_8px_rgba(16,185,129,0.6)]" 
          : status === "processing" ? "drop-shadow-[0_0_8px_rgba(56,189,248,0.6)]"
          : status === "error" ? "drop-shadow-[0_0_8px_rgba(244,63,94,0.6)]"
          : ""
        }`}>
          <img 
            src="/contextflow.svg" 
            alt="ContextFlow Logo" 
            className={`w-full h-full object-cover transition-all duration-300 ${
              status === "processing" ? "hue-rotate-60 brightness-110" 
              : status === "error" ? "hue-rotate-180 brightness-110"
              : ""
            }`} 
          />
        </div>

        {/* Middle Audio Visualizer Dots */}
        <div className={`absolute left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 flex gap-[5px] items-center justify-center h-6 transition-opacity duration-300 ${isExpanded ? "opacity-100" : "opacity-0"}`}>
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
