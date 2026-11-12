# AsteroidOS Qt6/QML presenter (planned)

Phase 6 exit is a hello Contract app on an AsteroidOS image, driven headless.
This directory will hold the thin QML shell that:

1. Creates a `QQuickView` (or AsteroidOS launcher integration) at watch resolution.
2. Feeds frames from `exact_linux` paint (same path as headless smoke today).
3. Forwards touch through the Linux host input seam once evdev is on device.

## Blockers on macOS (honest)

- Full Qt6 cross link from aarch64-darwin is partially supported in
  `github.com/Wawona/AsteroidOS-SDK` (Gtk/Qt overlay drops multimedia on Darwin).
- Product Rust for AsteroidOS builds with `bun host/asteroid/build.mjs` against
  `armv7-unknown-linux-gnueabihf`; QEMU user-mode smoke runs on Linux hosts only.
- Until QML lands, use `bun scripts/build-asteroid.mjs hello` (host build) and
  `bun scripts/agent.mjs asteroid --app hello tree`.

Canonical issue: [Wawona/Wawona#246](https://github.com/Wawona/Wawona/issues/246).
