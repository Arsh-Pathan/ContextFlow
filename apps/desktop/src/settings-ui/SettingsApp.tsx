/**
 * Settings window root. Full UI is built in the next phase; this is the shell
 * that the Tauri window loads so window wiring can be verified independently.
 */
export function SettingsApp() {
  return (
    <div className="w-screen h-screen flex items-center justify-center bg-cf-bg text-cf-text">
      <p className="text-cf-muted">ContextFlow Settings</p>
    </div>
  );
}
