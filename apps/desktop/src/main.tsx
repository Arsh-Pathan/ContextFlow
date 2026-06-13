import React from "react";
import ReactDOM from "react-dom/client";

import { App } from "./App";
import { SettingsApp } from "./settings-ui/SettingsApp";
import { SettingsProvider } from "./settings";
import "./styles.css";

/**
 * One bundle serves two Tauri windows. The settings window is opened with
 * `index.html?window=settings`; everything else is the floating bubble.
 * We branch on that query param to mount the right surface, and tag <html>
 * so window-scoped CSS (e.g. the settings window's opaque canvas) can apply.
 */
function resolveWindow(): "settings" | "bubble" {
  const param = new URLSearchParams(window.location.search).get("window");
  return param === "settings" ? "settings" : "bubble";
}

const root = document.getElementById("root");
if (!root) {
  throw new Error("ContextFlow: missing #root element");
}

const surface = resolveWindow();
document.documentElement.setAttribute("data-cf-window", surface);

ReactDOM.createRoot(root).render(
  <React.StrictMode>
    <SettingsProvider>
      {surface === "settings" ? <SettingsApp /> : <App />}
    </SettingsProvider>
  </React.StrictMode>,
);
