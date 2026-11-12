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
- `bun scripts/build-asteroid.mjs hello`: green (Bun 1.4.2). Binary:
  `target/host-dev/hello-asteroid`.
- `EXACT_AGENT=1 bun scripts/agent.mjs asteroid --app hello tree state`: green.
  Tree: `Text#1 "Hello world"` at 240×240 headless cpu painter.
- `scripts/help.mjs` gained an `asteroid` verb so `guardFlags('asteroid', …)` works.
- `host/asteroid/build.mjs` uses `nix build --print-out-paths` (Determinate Nix 2.35).

## Cross / QEMU

- `bun host/asteroid/build.mjs hello` with `ASTEROIDOS_SDK`: nixpkgs cross
  `toolchain` still compiling on Darwin (gettext and friends for armv7hf).
- macOS path verifies armv7 ELF with `file` once linked (no qemu-arm on Darwin).
- Full watch image / Qt6/QML presenter still planned (`host/asteroid/qt/README.md`).

## Exit criteria status

Issue #246 exit is a hello Contract on an AsteroidOS image or device. This
machine's deliverable is the headless agent seam (tree/state green). Armv7
cross and on-image Qt remain follow-ups when the SDK toolchain finishes.

Recheck 2026-11-11 evening: `EXACT_AGENT=1 bun scripts/agent.mjs asteroid
--app hello tree state` still prints `Text#1 "Hello world"` at 240×240.

## Armv7 cross (2026-11-12)

- AsteroidOS-SDK `#toolchain` built on Darwin
  (`armv7l-unknown-linux-gnueabihf-gcc` wrappers).
- `host/asteroid/build.mjs` maps Rust target `armv7-unknown-linux-gnueabihf` to
  the GNU `armv7l-…` linker prefix.
- `ASTEROIDOS_SDK=… bun host/asteroid/build.mjs hello` →
  `target/armv7-unknown-linux-gnueabihf/host-dev/hello-asteroid` (ELF 32-bit
  ARM EABI5). `file` confirms; QEMU user-mode remains Linux-host-only.
