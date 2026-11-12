// The Apple TV smoke: `bun scripts/smoke.mjs tvos [--app caltrain] [--shot <png>]`. Drives
// Caltrain on an Apple TV simulator through the iOS presenter, which tvOS builds with
// (`bun host/apple/build.mjs --tvos caltrain`; `--sim`/EXACT_SIM picks the simulator):
// the home screen, `check` in both schemes, the station search through the host's input
// path, and the clock moving a countdown. With no Apple TV
// simulator it prints `tvos: unsupported: …` and exits 0, the convention for a missing carrier.
import { randomUUID } from 'node:crypto';
import { simulators } from '../host/apple/devices.mjs';
import { open as openAgent } from './agent.mjs';
import { checkOp } from './agent-check.mjs';

const byTestId = (t, id) => t.nodes.find((n) => n.props.testId === id);

/** Run the drive; `check(ok, what)` records a failure. Returns 'unsupported' without an Apple TV simulator. */
export async function tvosSmoke({ app = 'caltrain', shot, check }) {
  if (!process.env.EXACT_SIM && !simulators().some((d) => /SimRuntime\.tvOS/.test(d.runtime))) {
    console.log('tvos: unsupported: no Apple TV simulator (Xcode › Settings › Components installs the tvOS runtime)');
    return 'unsupported';
  }
  const s = await openAgent({ host: 'tvos', app, storage: `smoke-${randomUUID()}` });
  try {
    let tree = await s.tree();
    check(byTestId(tree, 'home-screen') && byTestId(tree, 'station-name'), 'tvos: the home screen did not boot');
    const { check: runs } = await checkOp(s, ['both']);
    for (const r of runs) check(r.findings.length === 0, `tvos check ${r.scheme}: ${r.findings.map((f) => f.line).join('; ')}`);

    await s.tap('change-station');
    tree = await s.tree();
    check(byTestId(tree, 'stations-screen'), 'tvos: Change station did not open the stations screen');
    await s.type('station-search', 'Palo');
    // Typing brings up tvOS's full-screen keyboard over the list; its Done (Return) puts it away.
    await s.type('station-search', { key: 'Enter' });
    await s.clock('settle');
    tree = await s.tree();
    const matches = tree.nodes.filter((n) => n.type === 'Pressable' && n.props.testId?.startsWith('station-'));
    check(matches.length === 1 && matches[0].props.testId === 'station-paloalto', `tvos: the search shows ${matches.map((m) => m.props.testId).join(', ') || 'nothing'}, not station-paloalto alone`);
    await s.tap('station-paloalto');
    tree = await s.tree();
    check(byTestId(tree, 'station-name')?.props.text === 'Palo Alto', `tvos: after picking Palo Alto the station is ${byTestId(tree, 'station-name')?.props.text}`);

    check(byTestId(tree, 'home-screen'), 'tvos: picking a station did not return home');

    const countdown = () => tree.nodes.find((n) => n.props.testId?.startsWith('countdown-'))?.props.text;
    const before = countdown();
    await s.clock('+60000');
    tree = await s.tree();
    check(before != null && countdown() !== before, `tvos: a minute on the clock left the first countdown at ${before}`);
    if (shot) console.log(JSON.stringify(await s.screenshot(shot)));
  } finally { await s.close(); }
  return 'ok';
}
