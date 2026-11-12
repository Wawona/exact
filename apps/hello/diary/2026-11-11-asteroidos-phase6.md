# AsteroidOS Phase 6 scaffold (2026-11-11)

## Done

- `host/asteroid` (`exact-asteroid`): wraps `exact_linux` headless path, 240×240 default
  viewport, `EXACT_AGENT_HOST=asteroid` for agent refusals.
- `apps/hello/asteroid` (`hello-asteroid`): hello Contract baked for platform `asteroid`.
- `scripts/build-asteroid.mjs`, `host/asteroid/build.mjs` (cross via AsteroidOS-SDK flake).
- `docs/platforms.md` row + AsteroidOS section; manifest/schema/shells/agent wiring.

## Proof attempted (macOS agent host)

- `cargo build --profile host-dev -p exact-asteroid`: **green**.
- `cargo build --profile host-dev -p hello-asteroid`: **blocked** in this environment:
  build.rs compatibility id calls shader inventory; **bun not on PATH** (`BUN` unset).
  AGENTS.md expects pinned Bun; install before `bun scripts/build-asteroid.mjs hello`
  and `bun scripts/agent.mjs asteroid --app hello tree`.

## Cross / QEMU

- `nix build ../AsteroidOS-SDK#cross-qemu-smoke`: started; macOS path verifies armv7 ELF
  only (no qemu-arm in nixpkgs on Darwin per SDK README).

## Still open (exit criteria)

- Qt6/QML presenter linked on watch image (`host/asteroid/qt/README.md`).
- hello binary on AsteroidOS device or image under `EXACT_AGENT=1`.
- Full armv7 link of `hello-asteroid` after Bun + `rustup target add armv7-unknown-linux-gnueabihf`.
