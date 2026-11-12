//! Wawona Exact chrome on Windows.

#![cfg_attr(
    all(target_os = "windows", not(debug_assertions)),
    windows_subsystem = "windows"
)]
#![deny(missing_docs)]

/// The baked plan, written by `build.rs`.
const PLAN: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/app.plan"));

/// Compatibility id beside the plan.
const COMPAT: &str = include_str!(concat!(env!("OUT_DIR"), "/compat.json"));

include!(concat!(env!("OUT_DIR"), "/window.rs"));

fn main() {
    std::process::exit(exact_windows::run::<wawona_data::Wawona>(
        "Wawona",
        window(),
        PLAN,
        COMPAT,
    ));
}
