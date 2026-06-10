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

  // Smooth incoming level with exponential moving average
  const smoothLevelRef = useRef(0);
  const raw = level ?? 0;
  // Heavy smoothing — takes ~20 frames (400ms) to reach 95% of a step
  smoothLevelRef.current = smoothLevelRef.current * 0.88 + raw * 0.12;
  const smoothed = smoothLevelRef.current;

  // 6 dots for the visualizer
  const numDots = 6;
  const dots = Array.from({ length: numDots }).map((_, i) => {
    let scaleY = 1;

    if (status === "listening" && level !== undefined) {
      const distanceToCenter = Math.abs((numDots - 1) / 2 - i);
      const curve = 1 - (distanceToCenter / ((numDots - 1) / 2)) * 0.35;

      // Gentle gain — just enough to show movement without jitter
      const baseGain = smoothed * 6;

      scaleY = 1 + Math.min(1.6, baseGain * curve);
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
      transform: status === "listening" ? `scaleY(${scaleY})` : undefined,
      animation: animation,
    };

    const transitionClass = status === "listening"
      ? "transition-transform duration-150 ease-out"
      : "transition-all duration-300 ease-in-out";

    return (
      <div
        key={i}
        className={`w-1.5 rounded-full origin-center ${transitionClass} ${
          status === "processing" 
            ? "h-1.5 bg-sky-400 shadow-[0_0_8px_rgba(56,189,248,0.8)]" 
            : status === "error"
              ? "h-1.5 bg-rose-500"
              : status === "listening"
                ? "h-2 bg-white shadow-[0_0_12px_rgba(255,255,255,1)]"
                : "h-1.5 bg-gray-500"
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
    gradientStyle = "conic-gradient(from 0deg, transparent 0%, transparent 30%, #0e7490 40%, #10b981 48%, #ffffff 50%, transparent 50%, transparent 80%, #0e7490 90%, #10b981 98%, #ffffff 100%)";
    spinDuration = "2.5s";
  } else if (status === "processing") {
    shadowClass = "shadow-[0_0_24px_rgba(14,116,144,0.5)]";
    gradientStyle = "conic-gradient(from 0deg, transparent 0%, transparent 30%, #0ea5e9 40%, #38bdf8 48%, #ffffff 50%, transparent 50%, transparent 80%, #0ea5e9 90%, #38bdf8 98%, #ffffff 100%)";
    spinDuration = "1.5s";
  } else if (status === "error") {
    shadowClass = "shadow-[0_0_24px_rgba(244,63,94,0.4)]";
    gradientStyle = "conic-gradient(from 0deg, transparent 0%, transparent 30%, #be123c 40%, #f43f5e 48%, #ffffff 50%, transparent 50%, transparent 80%, #be123c 90%, #f43f5e 98%, #ffffff 100%)";
    spinDuration = "0s"; // Stop spinning on error
  }

  const isExpanded = status !== "idle";
  const wrapperClass = isExpanded ? "w-[180px] opacity-100" : "w-[44px] opacity-0";

  return (
    <div
      role="status"
      aria-label={ariaLabel}
      title={tooltip}
      className={`relative h-[44px] rounded-[22px] overflow-hidden p-[1.5px] transition-all duration-400 ease-[cubic-bezier(0.16,1,0.3,1)] ${wrapperClass} ${shadowClass}`}
      data-status={status}
    >
      <style>{`
        @keyframes processing-wave {
          0%, 100% { transform: translateY(0); opacity: 0.4; }
          25% { transform: translateY(-12px); opacity: 1; }
          75% { transform: translateY(12px); opacity: 1; }
        }
        @keyframes idle-breathe {
          0%, 100% { opacity: 0.3; transform: scale(1); }
          50% { opacity: 0.7; transform: scale(1.1); }
        }
        @keyframes error-shake {
          0%, 100% { transform: translateX(0); }
          25% { transform: translateX(-2px); }
          75% { transform: translateX(2px); }
        }
        @keyframes spin-gradient {
          from { transform: rotate(0deg); }
          to { transform: rotate(360deg); }
        }
      `}</style>

      {/* Spinning Gradient Border */}
      <div 
        className="absolute -inset-[150%] rounded-full opacity-90"
        style={{
          background: gradientStyle,
          animation: spinDuration === "0s" ? "none" : `spin-gradient ${spinDuration} linear infinite`
        }}
      />

      {/* Inner Content Box */}
      <div className="relative w-full h-full rounded-[20px] bg-[#1a1a1a]">
        
        {/* Left Icon (Logo) */}
        <div className={`absolute left-2.5 top-1/2 -translate-y-1/2 flex items-center justify-center w-7 h-7 rounded-full overflow-hidden transition-all duration-300 ${
          status === "listening" ? "drop-shadow-[0_0_8px_rgba(16,185,129,0.6)] scale-110" 
          : status === "processing" ? "drop-shadow-[0_0_8px_rgba(56,189,248,0.6)]"
          : status === "error" ? "grayscale"
          : ""
        }`}>
          <img src="/contextflow.svg" alt="ContextFlow Logo" className="w-full h-full object-cover" />
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
