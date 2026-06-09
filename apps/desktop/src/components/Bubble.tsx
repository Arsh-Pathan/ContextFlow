import type { CSSProperties } from "react";
import { Ear, X } from "lucide-react";
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
    // Basic scaling based on the live audio level. 
    // We add a little random/stagger effect based on the index.
    let scale = 1;
    if (status === "listening" && level !== undefined) {
      const staggeredLevel = Math.min(1, level * (1.5 + (i % 2) * 1.5));
      scale = 1 + staggeredLevel * 1.5;
    } else if (status === "processing") {
      // Pulse sequence for processing
      scale = 1; // Handled by Tailwind classes
    }

    const delay = `${i * 150}ms`;
    
    return (
      <div
        key={i}
        className={`w-1.5 h-1.5 rounded-full transition-transform duration-75 ease-out ${
          status === "processing" 
            ? "bg-sky-400 animate-pulse" 
            : status === "error"
              ? "bg-rose-500"
              : status === "listening"
                ? "bg-gray-200"
                : "bg-gray-500"
        }`}
        style={{
          transform: `scale(${scale})`,
          animationDelay: status === "processing" ? delay : "0ms",
        }}
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
      className="w-full h-full flex items-center justify-between px-3 rounded-[22px] bg-[#222222] border-2 border-[#333333] shadow-[0_4px_24px_rgba(0,0,0,0.5)] overflow-hidden"
      data-status={status}
    >
      {/* Left Icon (Ear) */}
      <div className={`flex items-center justify-center w-8 h-8 ${
        status === "listening" ? "text-pink-400 drop-shadow-[0_0_8px_rgba(244,114,182,0.5)]" 
        : status === "error" ? "text-rose-500"
        : status === "processing" ? "text-sky-400"
        : "text-gray-500"
      }`}>
        <Ear size={20} />
      </div>

      {/* Middle Audio Visualizer Dots */}
      <div className="flex gap-1.5 items-center justify-center flex-1 mx-2">
        {dots}
      </div>

      {/* Right Icon (Close Button) */}
      <button 
        onClick={handleClose}
        className="flex items-center justify-center w-6 h-6 rounded-full bg-[#3a3a3a] text-gray-300 hover:text-white hover:bg-[#4a4a4a] transition-colors"
      >
        <X size={14} strokeWidth={3} />
      </button>
    </div>
  );
}
