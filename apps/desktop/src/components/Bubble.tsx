import type { CSSProperties } from "react";
import { X } from "lucide-react";
import { getCurrentWindow } from "@tauri-apps/api/window";

export type DictationStatus = "idle" | "listening" | "processing" | "error";

interface BubbleProps {
  status: DictationStatus;
  level?: number;
  message?: string;
}

const STATUS_LABEL: Record<DictationStatus, string> = {
  idle: "Idle",
  listening: "Listening",
  processing: "Processing",
  error: "Error",
};

export function Bubble({ status, level, message }: BubbleProps) {
  const ariaLabel =
    status === "error" && message
      ? `ContextFlow: Error — ${message}`
      : `ContextFlow: ${STATUS_LABEL[status]}`;

  // 6 dots for the visualizer
  const numDots = 6;
  const dots = Array.from({ length: numDots }).map((_, i) => {
    // Determine dynamic scale for audio reactivity
    let scaleY = 1;
    let translateY = 0;
    
    // Very reactive audio profile
    if (status === "listening" && level !== undefined) {
      const distanceToCenter = Math.abs((numDots - 1) / 2 - i);
      const intensity = 1 - (distanceToCenter / ((numDots - 1) / 2)) * 0.4;
      const baseGain = level * 10;
      scaleY = 1 + Math.min(3.5, baseGain * intensity);
    }

    const delayMs = i * 100;
    const delay = `${delayMs}ms`;
    
    // Determine the animation based on state
    let animation = 'none';
    if (status === "processing") {
      animation = `processing-bounce 0.8s ease-in-out ${delay} infinite`;
    } else if (status === "idle") {
      animation = `idle-breathe 2s ease-in-out ${delayMs * 2}ms infinite`;
    } else if (status === "error") {
      animation = `error-shake 0.4s ease-in-out ${delay} infinite`;
    }
      
    const dotStyle: CSSProperties = {
      transform: status === "listening" ? `scaleY(${scaleY})` : `scaleY(1) translateY(${translateY}px)`,
      animation: animation,
    };

    // Use a fast transition for listening so it feels snappy and reactive,
    // but a slower transition for other state changes.
    const transitionClass = status === "listening" 
      ? "transition-transform duration-75 ease-out" 
      : "transition-all duration-300 ease-in-out";

    return (
      <div
        key={i}
        className={`w-1.5 rounded-full origin-center ${transitionClass} ${
          status === "processing" 
            ? "h-1.5 bg-sky-400" 
            : status === "error"
              ? "h-1.5 bg-rose-500"
              : status === "listening"
                ? "h-2 bg-white shadow-[0_0_8px_rgba(255,255,255,0.8)]"
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

  return (
    <div
      role="status"
      aria-label={ariaLabel}
      title={status === "error" && message ? message : STATUS_LABEL[status]}
      className="w-full h-full flex items-center justify-between px-3 rounded-[22px] bg-[#1a1a1a] border border-[#2a2a2a] shadow-[0_4px_24px_rgba(0,0,0,0.6)] overflow-hidden"
      data-status={status}
    >
      <style>{`
        @keyframes processing-bounce {
          0%, 100% { transform: translateY(0) scale(1); opacity: 0.4; }
          50% { transform: translateY(-4px) scale(1.2); opacity: 1; }
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
      `}</style>
      
      {/* Left Icon (Logo) */}
      <div className={`flex items-center justify-center w-7 h-7 rounded-full overflow-hidden transition-all duration-300 ${
        status === "listening" ? "drop-shadow-[0_0_8px_rgba(16,185,129,0.6)] scale-110" 
        : status === "processing" ? "drop-shadow-[0_0_8px_rgba(56,189,248,0.6)] animate-pulse"
        : status === "error" ? "grayscale"
        : ""
      }`}>
        <img src="/contextflow.svg" alt="ContextFlow Logo" className="w-full h-full object-cover" />
      </div>

      {/* Middle Audio Visualizer Dots */}
      <div className="flex gap-[5px] items-center justify-center flex-1 mx-2 h-6">
        {dots}
      </div>

      {/* Right Icon (Close Button) */}
      <button 
        onClick={handleClose}
        className="flex items-center justify-center w-6 h-6 rounded-full bg-white/5 text-gray-400 hover:text-white hover:bg-white/15 transition-all duration-200"
      >
        <X size={14} strokeWidth={2.5} />
      </button>
    </div>
  );
}
