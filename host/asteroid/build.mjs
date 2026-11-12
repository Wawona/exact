#!/usr/bin/env bun
// Cross-build an app's AsteroidOS binary (armv7hf) when the AsteroidOS-SDK flake is available.
//
//   bun host/asteroid/build.mjs <app> [--host-only]
//
// `--host-only` builds the same crate for the build machine (headless agent on macOS).
import { spawnSync } from 'node:child_process';
import { existsSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { HOST_DEV, asteroidBuild, cargoEnvironment, resolveApp } from '../../scripts/app.mjs';

const ROOT = resolve(fileURLToPath(new URL('../..', import.meta.url)));
// Rust target triple (no "l"). AsteroidOS-SDK wrappers use armv7l-… for the
// GNU toolchain prefix; map that when setting CC/CXX/AR/linker.
const TARGET = 'armv7-unknown-linux-gnueabihf';
const GNU_PREFIX = 'armv7l-unknown-linux-gnueabihf';
const defaultSdk = resolve(ROOT, '..', 'AsteroidOS-SDK');

const [name, ...rest] = process.argv.slice(2);
if (!name || name.startsWith('-')) {
  console.error(`usage: bun host/asteroid/build.mjs <app> [--host-only]`);
  process.exit(2);
}
const hostOnly = rest.includes('--host-only');
const app = resolveApp(name);

if (hostOnly) {
  const built = spawnSync('cargo', asteroidBuild(app).slice(1), {
    cwd: app.workspace ?? ROOT,
    env: cargoEnvironment({ ...process.env, CARGO_TARGET_DIR: app.target }),
    stdio: 'inherit',
  });
  process.exit(built.status ?? 1);
}

const sdkFlake = process.env.ASTEROIDOS_SDK ?? defaultSdk;
if (!existsSync(resolve(sdkFlake, 'flake.nix'))) {
  console.error(`ASTEROIDOS_SDK must point at AsteroidOS-SDK (missing flake at ${sdkFlake})`);
  process.exit(1);
}

// Determinate Nix 2.35+: `--print-out-paths` (plural). Older nix had `--print-out-path`.
const toolchain = spawnSync('nix', ['build', `${sdkFlake}#toolchain`, '--no-link', '--print-out-paths'], {
  encoding: 'utf8',
});
if (toolchain.status !== 0) {
  console.error(toolchain.stderr || toolchain.stdout || 'nix build toolchain failed');
  process.exit(toolchain.status ?? 1);
}
const tc = toolchain.stdout.trim().split(/\s+/).filter(Boolean).at(-1) ?? '';
const gcc = `${tc}/bin/${GNU_PREFIX}-gcc`;
if (!existsSync(gcc)) {
  console.error(`cross gcc missing at ${gcc}`);
  process.exit(1);
}

const env = cargoEnvironment({
  ...process.env,
  CARGO_TARGET_DIR: app.target,
  CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_LINKER: gcc,
  CC_armv7_unknown_linux_gnueabihf: gcc,
  CXX_armv7_unknown_linux_gnueabihf: `${tc}/bin/${GNU_PREFIX}-g++`,
  AR_armv7_unknown_linux_gnueabihf: `${tc}/bin/${GNU_PREFIX}-ar`,
});

const args = [
  'build',
  ...asteroidBuild(app).slice(2),
  '--target',
  TARGET,
];
const built = spawnSync('cargo', args, { cwd: app.workspace ?? ROOT, env, stdio: 'inherit' });
if (built.status !== 0) process.exit(built.status ?? 1);

const bin = resolve(app.target, TARGET, HOST_DEV, app.crate('asteroid'));
console.log(bin);
process.exit(0);
