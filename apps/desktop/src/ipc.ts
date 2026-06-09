/**
 * Hand-mirrored counterpart of `crates/ipc-contracts/src/events.rs`.
 *
 * Both files declare the same shape; a future commit hooks `specta-typescript`
 * so this file is generated. Until then we keep them in sync manually — the
 * surface is small enough for that to be safe.
 */

import { listen, type UnlistenFn } from "@tauri-apps/api/event";

import type { DictationStatus } from "./components/Bubble";

/** Topic for the dictation status event. Must match `EVENT_DICTATION_STATUS` in Rust. */
export const EVENT_DICTATION_STATUS = "dictation://status";

/**
 * Payload of `EVENT_DICTATION_STATUS`. All optional fields are present
 * only when relevant — `level` during `listening`, `message` during
 * `error`, `provider` at startup, `warning` on non-fatal fallback.
 * The UI must tolerate any of them being absent.
 */
export interface DictationStatusEvent {
  status: DictationStatus;
  /** RMS in 0..=1, populated during `listening`. */
  level?: number;
  /** One-line user-facing context, populated during `error`. */
  message?: string;
  /** Provider identifier, set once at startup (e.g. "whisper-cpp", "windows-sr"). */
  provider?: string;
  /** Non-fatal advisory shown alongside any status. */
  warning?: string;
}

/**
 * Subscribe to dictation status updates from the Rust shell.
 *
 * The callback receives the full event payload so the bubble can react to
 * `level` (drive the listening pulse from real audio) and `message` (show
 * an error tooltip). Returns a Promise that resolves with an `unlisten`
 * function — call it during component cleanup to drop the subscription.
 */
export async function subscribeDictationStatus(
  onEvent: (event: DictationStatusEvent) => void,
): Promise<UnlistenFn> {
  return listen<DictationStatusEvent>(EVENT_DICTATION_STATUS, (event) => {
    onEvent(event.payload);
  });
}
