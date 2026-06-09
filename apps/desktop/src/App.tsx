/**
 * Root component of the ContextFlow desktop UI.
 *
 * The app currently hosts a single window — the floating dictation bubble.
 * Slice 6 adds a separate settings window; until then the bubble is the
 * entire UI surface.
 *
 * Bubble state is driven by `dictation://status` events emitted by the
 * Rust shell (and, in commit 7/7, the dictation orchestrator). The event
 * carries the status plus an optional live RMS level (`listening`) and
 * an optional error message (`error`). We thread both through to the
 * bubble so it reacts to real audio and shows error context on hover.
 *
 * The dev-only `window.__contextflow_setStatus` helper is preserved so the
 * visuals can be exercised standalone in a browser tab (without the Rust
 * pipeline) during UI iteration. It now accepts an optional level/message
 * so we can demo the listening pulse and error tooltips locally too.
 */

import { useEffect, useState } from "react";
import { getCurrentWindow, LogicalPosition, currentMonitor } from "@tauri-apps/api/window";

import { Bubble, type DictationStatus } from "./components/Bubble";
import { subscribeDictationStatus, type DictationStatusEvent } from "./ipc";

interface BubbleState {
  status: DictationStatus;
  level?: number;
  message?: string;
  provider?: string;
  warning?: string;
}

const INITIAL_STATE: BubbleState = { status: "idle" };

export function App() {
  const [state, setState] = useState<BubbleState>(INITIAL_STATE);

  // Subscribe to dictation status events from the Rust shell.
  // `subscribeDictationStatus` resolves to an `unlisten` function we call on
  // unmount to drop the Tauri event subscription.
  useEffect(() => {
    let unlisten: (() => void) | null = null;
    let active = true;

    // Build state with conditional spreads so we don't pass `undefined`
    // explicitly — the project's tsconfig has `exactOptionalPropertyTypes`
    // turned on, which treats `{ x: undefined }` and `{}` as different.
    subscribeDictationStatus((event: DictationStatusEvent) => {
      setState({
        status: event.status,
        ...(event.level !== undefined && { level: event.level }),
        ...(event.message !== undefined && { message: event.message }),
        ...(event.provider !== undefined && { provider: event.provider }),
        ...(event.warning !== undefined && { warning: event.warning }),
      });
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
  // when iterating on styles without the Rust pipeline running. Accepts
  // the same shape as the Rust event so we can demo `listening` pulses
  // and `error` tooltips locally too.
  //
  // Examples (in the browser console):
  //   __contextflow_setStatus("listening", { level: 0.6 })
  //   __contextflow_setStatus("error",     { message: "microphone unavailable" })
  useEffect(() => {
    type DebugWindow = Window & {
      __contextflow_setStatus?: (
        status: DictationStatus,
        extras?: { level?: number; message?: string },
      ) => void;
    };
    (window as DebugWindow).__contextflow_setStatus = (status, extras) => {
      setState({
        status,
        ...(extras?.level !== undefined && { level: extras.level }),
        ...(extras?.message !== undefined && { message: extras.message }),
      });
    };
    return () => {
      delete (window as DebugWindow).__contextflow_setStatus;
    };
  }, []);

  // Center the window at the bottom of the screen on load
  useEffect(() => {
    const win = getCurrentWindow();
    currentMonitor().then((monitor) => {
      if (monitor) {
        const scaleFactor = monitor.scaleFactor;
        const logicalWidth = monitor.size.width / scaleFactor;
        const logicalHeight = monitor.size.height / scaleFactor;
        
        // Window is 180px wide. Bottom offset is 60px.
        const x = (logicalWidth - 180) / 2;
        const y = logicalHeight - 120 - 44; // 60px from bottom, 44px is height
        
        win.setPosition(new LogicalPosition(x, y)).catch(console.warn);
      }
    }).catch(console.warn);
  }, []);

  // Show the window only when active (listening, processing, error)
  // Hide it when idle to stay out of the user's way.
  useEffect(() => {
    const win = getCurrentWindow();
    if (state.status === "idle") {
      win.hide().catch((err) => console.warn("Failed to hide window:", err));
    } else {
      win.show().catch((err) => console.warn("Failed to show window:", err));
    }
  }, [state.status]);

  return (
    <div className="w-screen h-screen flex items-center justify-center">
      <Bubble
        status={state.status}
        {...(state.level !== undefined && { level: state.level })}
        {...(state.message !== undefined && { message: state.message })}
        {...(state.provider !== undefined && { provider: state.provider })}
        {...(state.warning !== undefined && { warning: state.warning })}
      />
    </div>
  );
}
