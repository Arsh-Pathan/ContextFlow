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

export interface DictationStatusEvent {
  status: DictationStatus;
}

/**
 * Subscribe to dictation status updates from the Rust shell.
 *
 * Returns a Promise that resolves with an `unlisten` function — call it
 * during component cleanup to drop the subscription.
 */
export async function subscribeDictationStatus(
  onStatus: (status: DictationStatus) => void,
): Promise<UnlistenFn> {
  return listen<DictationStatusEvent>(EVENT_DICTATION_STATUS, (event) => {
    onStatus(event.payload.status);
  });
}
