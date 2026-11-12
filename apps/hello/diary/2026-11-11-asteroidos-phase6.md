# AsteroidOS Phase 6 scaffold (2026-11-11)

## Done

- `host/asteroid` (`exact-asteroid`): wraps `exact_linux` headless path, 240×240 default
  viewport, `EXACT_AGENT_HOST=asteroid` for agent refusals.
- `apps/hello/asteroid` (`hello-asteroid`): hello Contract baked for platform `asteroid`
  with `deploy.store.asteroid` `"0"`.
- `scripts/build-asteroid.mjs`, `host/asteroid/build.mjs` (cross via AsteroidOS-SDK flake).
- Agent: `scripts/agent-stdio.mjs` treats asteroid like linux for `EXACT_ASSETS`.
- `docs/platforms.md` row + AsteroidOS section; manifest/schema/shells/agent wiring.

## Proven (macOS agent host)

- `bun scripts/build-asteroid.mjs hello` → `target/host-dev/hello-asteroid`.
- `EXACT_AGENT=1 bun scripts/agent.mjs asteroid --app hello tree` →
  `Text#1 "Hello world"` (240×240 headless).
- `nix build ../AsteroidOS-SDK#cross-qemu-smoke` (Darwin: armv7 ELF check).

## Still open (full Phase 6 exit)

- Qt6/QML presenter linked on watch image (`host/asteroid/qt/README.md`).
- Armv7 product binary on an AsteroidOS image under QEMU system or a device.
