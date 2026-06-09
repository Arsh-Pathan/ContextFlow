/**
 * Root component of the ContextFlow desktop UI.
 *
 * The app currently hosts a single window — the floating dictation bubble.
 * Slice 6 adds a separate settings window; until then the bubble is the
 * entire UI surface.
 *
 * Bubble state is driven by `dictation://status` events emitted by the
 * Rust shell. The dev-only `window.__contextflow_setStatus` helper is
 * preserved so the visuals can be exercised standalone in a browser tab
 * (without the Rust pipeline) during UI iteration.
 */

import { useEffect, useState } from "react";

import { Bubble, type DictationStatus } from "./components/Bubble";
import { subscribeDictationStatus } from "./ipc";

export function App() {
  const [status, setStatus] = useState<DictationStatus>("idle");

  // Subscribe to dictation status events from the Rust shell.
  // `subscribeDictationStatus` resolves to an `unlisten` function we call on
  // unmount to drop the Tauri event subscription.
  useEffect(() => {
    let unlisten: (() => void) | null = null;
    let active = true;

    subscribeDictationStatus((next) => {
      setStatus(next);
    })
      .then((fn) => {
        // If the component unmounted before the subscribe resolved, tear
        // down immediately. Avoids leaking the listener.
        if (active) {
          unlisten = fn;
        } else {
          fn();
        }
      })
      .catch((err: unknown) => {
        // Tauri isn't available when the UI is opened in a plain browser
        // (Vite dev preview). That's fine — the devtools helper below
        // still works.
        console.warn("ContextFlow: dictation status subscription failed:", err);
      });

    return () => {
      active = false;
      if (unlisten) {
        unlisten();
      }
    };
  }, []);

  // Devtools-only helper: lets us flip bubble visuals from the JS console
  // when iterating on styles without the Rust pipeline running.
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
