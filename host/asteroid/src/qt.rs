//! Qt6/QML presenter boundary (not wired in product binaries yet).
//!
//! Cross Qt on Darwin is limited (see AsteroidOS-SDK `flake.nix` overlays). The
//! first shipping path is headless `EXACT_AGENT=1` on armv7 Linux; the QML shell
//! loads the same baked plan once the SDK sysroot is on the watch image.

/// Presenter integration state for tooling and docs.
pub const PRESENTER: &str = "planned";
