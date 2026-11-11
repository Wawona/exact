// `build.mjs --test`: the Swift host tests (LLP 1033 D4a), kept apart from the build.
import { cpSync, mkdirSync, writeFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { HOST_DEV, appleCargoClaims, cargoLibraryTarget, claimBuildOutput, developmentBuildEnv, injectedProfiles, resolveApp } from '../../scripts/app.mjs';
import { appleArtifacts, appleBuildLock, iosTarget, iosTriple, macTriple, pkg, svgFilterLibrary, svgFilterLibraryName, tvosTarget } from './build.mjs';
import { composedModule, moduleSeam } from './compose.mjs';
import { writeDataKeys } from './data-keys.mjs';
import { simulator, simulators } from './devices.mjs';
import { compileFixtures, iosTestRuns, macTests, onlyClasses } from './xctest.mjs';

const root = resolve(new URL('../..', import.meta.url).pathname);

/** The simulator platform a UIKit test run targets: `--ios` an iPhone, `--tvos` an Apple TV. */
const uikit = (tv) => tv
  ? { flag: '--tvos', name: 'tvOS', target: tvosTarget, sdk: 'appletvsimulator', triple: 'arm64-apple-tvos17.0-simulator', deployment: 'TVOS_DEPLOYMENT_TARGET', runtime: /SimRuntime\.tvOS/, device: /^Apple TV 4K/ }
  : { flag: '--ios', name: 'iOS', target: iosTarget, sdk: 'iphonesimulator', triple: iosTriple, deployment: 'IPHONEOS_DEPLOYMENT_TARGET', runtime: /SimRuntime\.iOS/, device: /^iPhone \d+ Pro$/ };

/** `--test`: the Swift host tests. They link `ExactKit`,
 *  which links an app's archive, so cargo builds one first — Caltrain's by
 *  default, any app's by name. Not one of the six checks (`rules/RULES.md`
 *  caps those at six); run it when the host's own behaviour changes. With
 *  `--ios` (`--tvos`), all tests compiled for UIKit through xcodebuild on an iPhone
 *  (Apple TV) simulator: `--sim`/EXACT_SIM, else one that is not running (another
 *  session may be driving a booted one), shut down again only if booted here.
 *  Swift availability guards decide which shared methods run there. `--only`
 *  runs the test classes it names (`onlyClasses`). `hasStd`, `read`, `run` and `runApple` are build.mjs's. */
export async function test(args, { hasStd, read, run, runApple }) {
  const tv = args.includes('--tvos'), ios = tv || args.includes('--ios'), sim = uikit(tv);
  if (ios && !hasStd(sim.flag, sim.target)) return;
  const app = resolveApp(args.find((a, i) => !a.startsWith('--') && args[i - 1] !== '--sim' && args[i - 1] !== '--only'));
  const only = args.includes('--only') ? onlyClasses(args, resolve(root, 'host/apple/tests/ExactKitTests'), ios ? sim.name : 'macOS') : null;
  app.prepare?.();
  const crate = app.crate(ios ? 'ios' : 'macos');
  const release = appleBuildLock(app);
  const paths = appleArtifacts(app, { composition: 'embedded' });
  let cargoRelease;
  try {
    const cargoEnv = { ...developmentBuildEnv(), CARGO_TARGET_DIR: app.target, ...(ios ? {
      SDKROOT: read('xcrun', ['--sdk', sim.sdk, '--show-sdk-path']).stdout.trim(),
      MACOSX_DEPLOYMENT_TARGET: '14.0', [sim.deployment]: '17.0',
      // Build scripts compile Objective-C++ for the Mac; SDKROOT names the simulator's.
      HOST_CXXFLAGS: `${process.env.HOST_CXXFLAGS ?? ''} -isysroot ${read('xcrun', ['--sdk', 'macosx', '--show-sdk-path']).stdout.trim()}`,
    } : {}) };
    const metadata = read('cargo', ['metadata', '--no-deps', '--format-version', '1'], { cwd: app.workspace, env: cargoEnv });
    if (metadata.status !== 0) throw new Error(`cargo metadata: ${metadata.stderr}`);
    const package_ = JSON.parse(metadata.stdout).packages.find(p => p.name === crate);
    const unit = package_ && cargoLibraryTarget(package_);
    if (!unit) throw new Error(`Cargo has no library target for ${crate}`);
    cargoRelease = claimBuildOutput(app, appleCargoClaims(app, ios ? sim.target : 'host', [unit])[0]);
    // The development profile, as an app's build: `release` put the whole graph through LTO for a test run.
    run('cargo', ['rustc', '--crate-type', 'staticlib', ...injectedProfiles(app), '--profile', HOST_DEV, '-p', crate, '--lib', ...(ios ? ['--target', sim.target] : [])], { cwd: app.workspace, env: cargoEnv });
    const libDir = ios ? resolve(app.target, sim.target, HOST_DEV) : resolve(app.target, HOST_DEV);
    const env = { ...process.env, EXACT_TESTS: '1', EXACT_LIB_DIR: libDir, EXACT_LIB: unit.name.replace(/-/g, '_'), EXACT_APP_COMPOSITION: 'embedded' };
    mkdirSync(paths.namespace, { recursive: true });
    env.EXACT_SVG_METALLIB = svgFilterLibrary(ios ? sim.sdk : 'macosx', ios ? '17.0' : '14.0', resolve(paths.namespace, svgFilterLibraryName), true);
    if (ios) env.TEST_RUNNER_EXACT_SVG_METALLIB = env.EXACT_SVG_METALLIB;
    // The optional modules tests drive, built for the tests' platform as a build embeds them and named to the runner: the
    // video and WebKit arms and the Canvas 2D GPU module. A test without one is a missing prerequisite (Coverage.swift).
    const armDir = resolve(paths.namespace, 'arms');
    const arm = (name, sources, frameworks, out = resolve(armDir, `lib${name}.dylib`)) => (runApple('xcrun', ['--sdk', ios ? sim.sdk : 'macosx', 'swiftc', '-parse-as-library', '-emit-library', '-O', '-module-name', name,
      '-module-cache-path', resolve(armDir, 'cache'), ...sources, ...frameworks.flatMap(f => ['-framework', f]), '-target', ios ? sim.triple : `${macTriple}14.0`, '-o', out]), out);
    const named = ios ? 'TEST_RUNNER_' : '';
    env[`${named}EXACT_WIDGET_RELOAD_TEST`] = arm('WidgetReloadTest', [resolve(pkg, 'tests/ExactKitTests/WidgetContainerTests.swift'), '-D', 'EXACT_TEST_WIDGET_RELOAD'], ['Foundation']);
    env[`${named}EXACT_VIDEO_DYLIB`] = arm('ExactVideoArm', ['VideoArm.swift', 'NowPlaying.swift', 'PictureInPicture.swift'].map(f => resolve(root, 'host/apple/videoarm', f)), ['AVKit', 'MediaPlayer']);
    if (!ios) env.EXACT_MIDI_DYLIB = arm('ExactMIDIArm', [resolve(root, 'host/apple/midiarm/MIDIArm.swift'), '-import-objc-header', resolve(root, 'host/apple/midiarm/midi_ring.h')], ['CoreMIDI']); // MIDITests' loopback
    // tvOS has no WebKit.
    if (!tv) env[`${named}EXACT_TEST_WEB_MODULE`] = arm('ExactWebArm', ['WebArm.swift', 'FrameBrowser.swift', 'FrameWindows.swift', 'FrameCapture.swift', 'FramePDF.swift', 'FrameAutomation.swift', 'FrameFiles.swift', 'FrameInspector.swift', 'FramePrint.swift'].map(f => resolve(root, 'host/apple/webarm', f)).concat(['-D', 'EXACT_DEVELOPMENT']), ['WebKit', 'PDFKit']);
    // The GPU module goes to its own tests alone: loaded, every canvas draws on the GPU (xctest.mjs).
    const modules = resolve(process.env.CARGO_TARGET_DIR ?? resolve(root, 'target'), 'apple-modules');
    run('cargo', ['build', '--profile', HOST_DEV, '-p', 'exact-canvas-vello', '--lib', ...(ios ? ['--target', sim.target] : []), '--target-dir', modules, '--manifest-path', resolve(root, 'Cargo.toml')],
      { env: { ...cargoEnv, [`CARGO_PROFILE_${HOST_DEV.toUpperCase().replace(/-/g, '_')}_STRIP`]: 'false' } });
    const canvasGpu = resolve(modules, ...(ios ? [sim.target] : []), HOST_DEV, 'libexact_canvas_vello.dylib');
    run('cargo', ['build', '-q', '-p', 'contract', '--bin', 'contract', '--manifest-path', resolve(root, 'Cargo.toml')]);
    env.EXACT_CONTRACT = resolve(process.env.CARGO_TARGET_DIR || resolve(root, 'target'), 'debug', 'contract');
    if (!ios) {
      compileFixtures(root, run, env, paths.namespace, 'EXACT', app.id);
      const swiftEnv = { ...env, MACOSX_DEPLOYMENT_TARGET: '14.0' }, scratch = resolve(paths.namespace, 'tests');
      runApple('swift', ['build', '--build-tests', '--scratch-path', scratch], { cwd: pkg, stdio: 'inherit', env: swiftEnv });
      return await macTests(pkg, scratch, swiftEnv, canvasGpu, only);
    }
    // The fixture's plan and hatch module (LLP 1075.003 §3.9), built for the
    // tests' simulator so they run its hatches over its routes: the glue, its
    // typed keys, its Swift.
    const fixture = resolveApp('native-fixture'), fixtureDir = resolve(paths.namespace, 'fixture-module');
    writeDataKeys(fixture, resolve(fixtureDir, 'ExactDataKeys.swift'), 'ios');
    // Its Contract under the identity of the app the tests link, which the
    // runner requires of a plan (an app's plan boots in no other app).
    const source = resolve(fixtureDir, 'app');
    mkdirSync(source, { recursive: true });
    cpSync(resolve(fixture.dir, 'app.contract'), resolve(source, 'app.contract'));
    writeFileSync(resolve(source, 'app.json'), JSON.stringify({ ...fixture.manifest, $schema: undefined, id: app.id, app: { ...fixture.manifest.app, id: app.id } }));
    env.TEST_RUNNER_EXACT_FIXTURE_PLAN = resolve(fixtureDir, 'app.plan');
    run(env.EXACT_CONTRACT, ['build', resolve(source, 'app.contract'), '-o', env.TEST_RUNNER_EXACT_FIXTURE_PLAN]);
    // Its module is written for iOS and macOS (WebKit, context menus), and the tests that load it are iOS's.
    if (!tv) {
      env.TEST_RUNNER_EXACT_FIXTURE_MODULE = resolve(fixtureDir, 'libexact_modules.dylib');
      // The module composes no packages' extensions (LLP 1133.001 D5): their table, empty.
      writeFileSync(resolve(fixtureDir, 'ExactComposed.swift'), composedModule([], fixture.modules.apple.length > 0));
      runApple('xcrun', ['--sdk', sim.sdk, 'swiftc', '-parse-as-library', '-emit-library', '-O', '-swift-version', '5', '-module-name', 'ExactAppModules',
        '-module-cache-path', resolve(fixtureDir, 'cache'), ...moduleSeam(resolve, root), resolve(fixtureDir, 'ExactComposed.swift'), resolve(fixtureDir, 'ExactDataKeys.swift'),
        ...fixture.modules.apple, '-target', sim.triple, '-o', env.TEST_RUNNER_EXACT_FIXTURE_MODULE]);
    }
    compileFixtures(root, run, env, fixtureDir, 'TEST_RUNNER_EXACT', app.id);
    if (process.env.EXACT_REQUIRED) env.TEST_RUNNER_EXACT_REQUIRED = process.env.EXACT_REQUIRED;
    const pick = args.includes('--sim') ? args[args.indexOf('--sim') + 1] : process.env.EXACT_SIM;
    const before = simulators();
    const idle = before.filter(d => sim.runtime.test(d.runtime) && sim.device.test(d.name) && d.state !== 'Booted');
    const newest = (d) => Number(/OS-(\d+)-(\d+)/.exec(d.runtime)?.slice(1).join('.') ?? 0);
    const dev = simulator(pick ?? idle.sort((a, b) => newest(b) - newest(a))[0]?.udid, { hint: '--sim <udid|name> (or EXACT_SIM)' });
    const bootedHere = before.find(d => d.udid === dev.udid)?.state !== 'Booted';
    try {
      const destination = `platform=${sim.name} Simulator,id=${dev.udid}`, derived = resolve(paths.namespace, `${sim.name.toLowerCase()}-tests`), xcodeEnv = { ...env, [sim.deployment]: '17.0' };
      runApple('xcodebuild', ['build-for-testing', '-scheme', 'Exact-Package', '-destination', destination, '-derivedDataPath', derived], { cwd: pkg, stdio: 'inherit', env: xcodeEnv });
      iosTestRuns({ pkg, products: resolve(derived, 'Build/Products'), destination, udid: dev.udid, env: xcodeEnv, canvasGpu, hostSource: resolve(root, 'host/apple/testhost/TestHost.swift'), triple: sim.triple, runApple, run, only, tv });
    } finally { if (bootedHere) read('xcrun', ['simctl', 'shutdown', dev.udid]); }
  } finally { cargoRelease?.(); release(); }
}
