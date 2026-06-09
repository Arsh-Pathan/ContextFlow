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
    if (status === "listening" && level !== undefined) {
      // Create a smooth, organic audio reactivity profile across the dots
      // The middle dots will react more strongly than the outer dots
      const distanceToCenter = Math.abs((numDots - 1) / 2 - i);
      const intensity = 1 - (distanceToCenter / ((numDots - 1) / 2)) * 0.4;
      const baseGain = level * 4;
      
      scaleY = 1 + Math.min(1.5, baseGain * intensity);
    }

    // Delay for processing wave animation
    const delay = `${i * 100}ms`;
    
    // For processing state, we use an inline CSS animation to create a wave
    const processingAnimation = status === "processing" 
      ? `pulse-wave 1s ease-in-out ${delay} infinite`
      : 'none';
      
    // Combine inline styles
    const dotStyle: CSSProperties = {
      transform: status === "listening" ? `scaleY(${scaleY})` : 'scaleY(1)',
      animation: processingAnimation,
    };

    return (
      <div
        key={i}
        className={`w-1.5 rounded-full transition-transform duration-100 ease-out origin-center ${
          status === "processing" 
            ? "h-1.5 bg-sky-400" 
            : status === "error"
              ? "h-1.5 bg-rose-500"
              : status === "listening"
                ? "h-2 bg-white"
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
        @keyframes pulse-wave {
          0%, 100% { transform: scaleY(1); opacity: 0.5; }
          50% { transform: scaleY(2.5); opacity: 1; }
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
