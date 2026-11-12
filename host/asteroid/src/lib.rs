//! AsteroidOS host scaffolding (Phase 6).
//!
//! The runner, kernel, layout, and agent API run through [`exact_linux`] on a
//! round watch viewport. A Qt6/QML shell that embeds the same presenter is
//! planned; it is not linked yet (see `qt/README.md`).
//!
//! @ref Wawona/Wawona#246

#![deny(unsafe_code)]
#![deny(missing_docs)]

pub mod qt;

use exact_linux::{run_composed as linux_run, Composed};

/// Default headless viewport (AsteroidOS reference round displays).
pub const DEFAULT_VIEWPORT: (u32, u32) = (240, 240);

/// Mark this process for the agent API (`host/linux/src/agent.rs`).
pub fn mark_agent_host() {
    std::env::set_var("EXACT_AGENT_HOST", "asteroid");
}

/// Apply watch defaults when the environment does not name a size.
pub fn apply_watch_defaults() {
    if std::env::var_os("EXACT_SIZE").is_none() {
        std::env::set_var(
            "EXACT_SIZE",
            format!("{}x{}", DEFAULT_VIEWPORT.0, DEFAULT_VIEWPORT.1),
        );
    }
}

/// Run with the Linux host's composed entry (hatches, packages).
pub fn run_composed<D: exact_runner::DataSource + Default + 'static>(
    plan: &[u8],
    compat: &str,
    composed: Composed,
) -> i32 {
    mark_agent_host();
    apply_watch_defaults();
    linux_run::<D>(plan, compat, composed)
}

/// [`run_composed`] with nothing composed.
pub fn run<D: exact_runner::DataSource + Default + 'static>(plan: &[u8], compat: &str) -> i32 {
    run_composed::<D>(plan, compat, Composed::default())
}
