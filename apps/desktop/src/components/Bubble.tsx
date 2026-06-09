import type { CSSProperties } from "react";

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

// Extremely premium inner glow and border combinations
const OUTER_STYLES: Record<DictationStatus, string> = {
  idle: "border-white/10 shadow-[inset_0_0_12px_rgba(255,255,255,0.05),0_4px_12px_rgba(0,0,0,0.5)]",
  listening: "border-emerald-400/50 shadow-[inset_0_0_15px_rgba(52,211,153,0.4),0_0_20px_rgba(52,211,153,0.3)]",
  processing: "border-sky-400/50 shadow-[inset_0_0_15px_rgba(56,189,248,0.4),0_0_20px_rgba(56,189,248,0.3)] animate-pulse",
  error: "border-rose-500/50 shadow-[inset_0_0_15px_rgba(244,63,94,0.4),0_0_20px_rgba(244,63,94,0.3)]",
};

// Gorgeous vibrant gradients for the core
const CORE_STYLES: Record<DictationStatus, string> = {
  idle: "bg-gradient-to-tr from-white/10 to-white/30 backdrop-blur-md opacity-60",
  listening: "bg-gradient-to-tr from-emerald-500 to-teal-300 shadow-[0_0_20px_rgba(52,211,153,0.8)]",
  processing: "bg-gradient-to-tr from-sky-500 to-indigo-400 shadow-[0_0_20px_rgba(56,189,248,0.8)] animate-spin",
  error: "bg-gradient-to-tr from-rose-600 to-orange-400 shadow-[0_0_20px_rgba(244,63,94,0.8)]",
};

function listeningScale(level: number | undefined): number {
  if (level === undefined || level <= 0) return 1;
  const gained = Math.min(1, level * 2.5);
  return 1 + gained * 0.8;
}

export function Bubble({ status, level, message }: BubbleProps) {
  const dotStyle: CSSProperties =
    status === "listening" ? { transform: `scale(${listeningScale(level)})` } : {};

  const ariaLabel =
    status === "error" && message
      ? `ContextFlow: Error — ${message}`
      : `ContextFlow: ${STATUS_LABEL[status]}`;

  return (
    <div
      role="status"
      aria-label={ariaLabel}
      title={status === "error" && message ? message : STATUS_LABEL[status]}
      className={`relative w-10 h-10 flex items-center justify-center rounded-full border transition-all duration-300 ease-out bg-black/40 backdrop-blur-2xl ${OUTER_STYLES[status]}`}
      data-status={status}
    >
      {/* Decorative spinning ring for processing state */}
      {status === "processing" && (
        <div className="absolute inset-0 rounded-full border-t-2 border-r-2 border-sky-300/80 animate-[spin_1s_linear_infinite]" />
      )}
      
      {/* The inner core dot */}
      <div
        className={`w-3 h-3 rounded-full transition-all duration-75 ease-out ${CORE_STYLES[status]}`}
        style={dotStyle}
      />
    </div>
  );
}
