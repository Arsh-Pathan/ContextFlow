/**
 * The floating dictation bubble.
 *
 * Visual states (Slice 1):
 *   - idle       : subtle pulse, indicates the app is alive and listening for the hotkey
 *   - listening  : the user is holding the hotkey and audio is being captured
 *   - processing : audio capture ended, transcription / injection in flight
 *   - error      : last session failed; user can hover for details (Slice 5+)
 *
 * The bubble lives in a frameless, transparent, always-on-top Tauri window
 * sized to ~96 × 96 px. See `apps/desktop/src-tauri/tauri.conf.json`.
 */

export type DictationStatus = "idle" | "listening" | "processing" | "error";

interface BubbleProps {
  status: DictationStatus;
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

export function Bubble({ status }: BubbleProps) {
  return (
    <div
      role="status"
      aria-label={`ContextFlow: ${STATUS_LABEL[status]}`}
      className="bubble-surface w-10 h-10 flex items-center justify-center"
      data-status={status}
    >
      <div
        className={`w-3 h-3 rounded-full transition-colors duration-150 ${STATUS_STYLES[status]}`}
      />
    </div>
  );
}
