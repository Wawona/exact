# AsteroidOS Phase 6 scaffold (2026-11-11)

## Done

- `host/asteroid` (`exact-asteroid`): wraps `exact_linux` headless path, 240×240 default
  viewport, `EXACT_AGENT_HOST=asteroid` for agent refusals.
- `apps/hello/asteroid` (`hello-asteroid`): hello Contract baked for platform `asteroid`.
- `scripts/build-asteroid.mjs`, `host/asteroid/build.mjs` (cross via AsteroidOS-SDK flake).
- `docs/platforms.md` row + AsteroidOS section; manifest/schema/shells/agent wiring.
- `scripts/help.mjs` `asteroid` verb so `guardFlags('asteroid', …)` works.

## Proof (macOS agent host)

- `cargo build --profile host-dev -p exact-asteroid`: green.
- `bun scripts/build-asteroid.mjs hello`: green (Bun 1.4.2).
- `EXACT_AGENT=1 bun scripts/agent.mjs asteroid --app hello tree state`: green.
  Tree: `Text#1 "Hello world"` at 240×240 headless cpu painter.

## Cross / QEMU

- `nix build ../AsteroidOS-SDK#cross-qemu-smoke`: macOS path verifies armv7 ELF
  only (no qemu-arm in nixpkgs on Darwin per SDK README).
- Full watch image / Qt6/QML presenter still planned (`host/asteroid/qt/README.md`).

## Exit criteria status

Phase 6 exit is a hello Contract on an AsteroidOS image or device. Headless agent
proof on the build machine is landed; armv7 cross binary next when SDK toolchain
builds; Qt on-device UI follows the image.
