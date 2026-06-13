/**
 * Optional ambient backdrop for the settings window (feature-flagged, off by
 * default). Two slow theme-coloured blooms drift behind the content. Pure CSS,
 * pointer-events-none, and disabled by prefers-reduced-motion via styles.css.
 */
export function AmbientBackground() {
  return (
    <div className="pointer-events-none absolute inset-0 z-0 overflow-hidden" aria-hidden>
      <div
        className="absolute -left-1/4 -top-1/4 h-[60%] w-[60%] rounded-full blur-3xl"
        style={{
          background: "radial-gradient(circle, rgba(var(--cf-glow-rgb),0.18), transparent 70%)",
          animation: "cf-ambient-a 18s ease-in-out infinite",
        }}
      />
      <div
        className="absolute -right-1/4 bottom-[-20%] h-[55%] w-[55%] rounded-full blur-3xl"
        style={{
          background:
            "radial-gradient(circle, color-mix(in srgb, var(--cf-accent-2) 22%, transparent), transparent 70%)",
          animation: "cf-ambient-b 22s ease-in-out infinite",
        }}
      />
    </div>
  );
}
