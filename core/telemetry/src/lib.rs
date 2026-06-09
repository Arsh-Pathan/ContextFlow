//! ContextFlow telemetry.
//!
//! Configures `tracing` for the whole app: rolling JSON logs to
//! `%LOCALAPPDATA%\ContextFlow\logs\`, opt-in latency histograms, and
//! Crashpad integration. Telemetry is **off by default**; the user enables it
//! explicitly in settings.
//!
//! ## Status
//!
//! Slice 1 introduces the logging subscriber. Metrics and crash reporting
//! land in Slice 6.

use std::path::Path;

/// Install a global `tracing` subscriber suitable for development.
///
/// Production setup (JSON to a rotating file under `%LOCALAPPDATA%`) lands
/// alongside the Slice 1 desktop shell once it can resolve that path.
pub fn install_dev_subscriber() {
    use tracing_subscriber::{fmt, prelude::*, EnvFilter};

    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,contextflow=debug"));

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer().with_target(true))
        .init();
}

/// Placeholder for the production subscriber. Wired in Slice 6 with the
/// actual `%LOCALAPPDATA%` resolution + rolling file appender.
pub fn install_production_subscriber(_log_dir: &Path) {
    install_dev_subscriber();
}
