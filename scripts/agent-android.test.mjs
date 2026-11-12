// The Android build's refusal of a minimum Android its TypeScript's Hermes bundle cannot load on.
import { test } from 'bun:test';
import assert from 'node:assert/strict';
import { mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { resolve } from 'node:path';
import { androidMinimumApi, hermesFloorRefusal } from './agent-android.mjs';

const app = (minimumOS) => ({ name: 'floor', manifest: minimumOS ? { host: { android: { minimumOS } } } : {} });

test('the minimum API follows the manifest as the Gradle build maps it', () => {
  assert.equal(androidMinimumApi({}), 31);
  assert.equal(androidMinimumApi(app('10').manifest), 29);
  assert.equal(androidMinimumApi(app('11').manifest), 30);
  assert.equal(androidMinimumApi(app('9').manifest), undefined);
});

test('a TypeScript app below its Hermes bundle\'s API level is refused before it builds', () => {
  const hermes = mkdtempSync(resolve(tmpdir(), 'exact-hermes-'));
  try {
    writeFileSync(resolve(hermes, 'hermes-input-receipt.json'), JSON.stringify({ configure: ['-DANDROID_ABI=arm64-v8a', '-DANDROID_PLATFORM=android-30'] }));
    assert.match(hermesFloorRefusal(app('10'), hermes), /minimumOS "10" is API 29, .* built for API 30/);
    assert.equal(hermesFloorRefusal(app('11'), hermes), null);
    assert.equal(hermesFloorRefusal(app(), hermes), null);
    assert.equal(hermesFloorRefusal(app('10'), resolve(hermes, 'absent')), null, 'no receipt: nothing to compare');
  } finally { rmSync(hermes, { recursive: true, force: true }); }
});
