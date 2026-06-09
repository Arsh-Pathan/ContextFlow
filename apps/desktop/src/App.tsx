/**
 * Root component of the ContextFlow desktop UI.
 *
 * The app currently hosts a single window — the floating dictation bubble.
 * Slice 6 adds a separate settings window; until then the bubble is the
 * entire UI surface.
 *
 * The bubble is intentionally minimal: a status indicator and (later) a mic
 * meter. All real state lives in the Rust core; this component subscribes to
 * dictation events emitted by the shell.
 */

import { useEffect, useState } from "react";

import { Bubble, type DictationStatus } from "./components/Bubble";

export function App() {
  const [status, setStatus] = useState<DictationStatus>("idle");

  // Slice 1 wires this to a real Tauri event (`dictation://status`) emitted
  // from the hotkey + dictation engine. For now we expose a window-level
  // helper so the bubble visuals can be exercised standalone during
  // development without the Rust side running.
  useEffect(() => {
    type DebugWindow = Window & {
      __contextflow_setStatus?: (s: DictationStatus) => void;
    };
    (window as DebugWindow).__contextflow_setStatus = setStatus;
    return () => {
      delete (window as DebugWindow).__contextflow_setStatus;
    };
  }, []);

  return (
    <div className="w-screen h-screen flex items-center justify-center">
      <Bubble status={status} />
    </div>
  );
}
