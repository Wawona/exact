#!/usr/bin/env bun
// Build an app's AsteroidOS host for the agent (host machine, headless).
//   bun scripts/build-asteroid.mjs <app>   then: bun scripts/agent.mjs asteroid --app <app> …
import { spawnSync } from 'node:child_process';
import { asteroidBinary, asteroidBuild, bakeWatchEnv, cargoEnvironment, resolveApp } from './app.mjs';
import { guardFlags } from './help.mjs';

const [name] = guardFlags('asteroid', process.argv.slice(2));
if (!name) {
  console.error('usage: bun scripts/build-asteroid.mjs <app>   (then: bun scripts/agent.mjs asteroid --app <app> …)');
  process.exit(2);
}
const app = resolveApp(name);
const [cmd, ...args] = asteroidBuild(app);
const built = spawnSync(cmd, args, {
  cwd: app.workspace ?? app.dir,
  env: cargoEnvironment({ ...process.env, ...bakeWatchEnv(app), CARGO_TARGET_DIR: app.target }),
  stdio: 'inherit',
});
if (built.status === 0) console.log(asteroidBinary(app));
process.exit(built.status ?? 1);
