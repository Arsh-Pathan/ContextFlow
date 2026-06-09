import type { CSSProperties } from "react";

/**
 * The floating dictation bubble.
 *
 * Visual states (Slice 1):
 *   - idle       : subtle pulse, indicates the app is alive and listening for the hotkey
 *   - listening  : the user is holding the hotkey and audio is being captured;
 *                  the inner dot pulses with the live RMS level when provided
 *   - processing : audio capture ended, transcription / injection in flight
 *   - error      : last session failed; hover reveals the underlying cause
 *
 * Drives off `DictationStatusEvent` payloads from `dictation://status` — see
 * `ipc.ts`. `level` is honored during the `listening` state; it lifts the
 * inner dot's scale and brightness so the bubble visibly reacts to speech.
 *
 * The bubble lives in a frameless, transparent, always-on-top Tauri window
 * sized to 40 × 40 px. See `apps/desktop/src-tauri/tauri.conf.json`.
 */

export type DictationStatus = "idle" | "listening" | "processing" | "error";

interface BubbleProps {
  status: DictationStatus;
  /** Live RMS level in 0..=1; only used when `status === "listening"`. */
  level?: number;
  /** Tooltip text; populated by `error` events. */
  message?: string;
}

const STATUS_STYLES: Record<DictationStatus, string> = {
  idle: "bg-white/15 animate-pulse-slow",
  listening: "bg-emerald-400/80 shadow-[0_0_24px_4px_rgba(52,211,153,0.55)]",
  processing: "bg-sky-400/80 shadow-[0_0_24px_4px_rgba(56,189,248,0.55)] animate-pulse",
  error: "bg-rose-500/80 shadow-[0_0_24px_4px_rgba(244,63,94,0.55)]",
};

const STATUS_LABEL: Record<DictationStatus, string> = {
  idle: "Idle",
  listening: "Listening",
  processing: "Processing",
  error: "Error",
};

/**
 * Scale the inner dot from baseline 1.0 up to 1.6 at peak (RMS == 1.0).
 * Real speech rarely exceeds ~0.3 RMS, so we apply a gentle gain so the
 * effect is visible without a screaming mic.
 */
function listeningScale(level: number | undefined): number {
  if (level === undefined || level <= 0) return 1;
  const gained = Math.min(1, level * 3); // gain ×3 → roughly maps 0..0.33 to 0..1
  return 1 + gained * 0.6;
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
      className="bubble-surface w-10 h-10 flex items-center justify-center"
      data-status={status}
    >
      <div
        className={`w-3 h-3 rounded-full transition-transform duration-75 ease-out ${STATUS_STYLES[status]}`}
        style={dotStyle}
      />
    </div>
  );
}
