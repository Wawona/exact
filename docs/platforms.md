# Platforms

Which hosts run a Contract app, how each is built and driven, and its OS floor. This fork
(`Wawona/exact`) aims to run one Contract app on the whole Apple family, Android with
WearOS, Linux, Windows, the web and AsteroidOS. The phases and their exit criteria are
tracked on [Wawona/Wawona#237](https://github.com/Wawona/Wawona/issues/237); update a row
in the commit that changes it.

| Host | State | Build | Agent target | OS floor | Phase |
|---|---|---|---|---|---|
| Web | shipping | `bun host/web/build.mjs <app>` | `web` | evergreen browsers | |
| macOS | shipping | `bun host/apple/build.mjs --run` | `macos` | 14.0 (`host.macos.minimumOS`) | |
| iOS, iPadOS | shipping | `bun host/apple/build.mjs --ios --run` | `ios` | 17.0 (`host.ios.minimumOS`) | |
| Linux | shipping | `cargo build --profile host-dev -p <app>-linux` | `linux` | | |
| tvOS | partial: driven and smoked on the iOS presenter (`bun scripts/smoke.mjs tvos`); XCTests do not build yet; no delivery | `bun host/apple/build.mjs --tvos --run` | `tvos` | 17.0 (the iOS floor) | [1](https://github.com/Wawona/Wawona/issues/239) |
| Windows | partial: runs, answers `check`, refuses a stale build; no delivery | `bun host/windows/build.mjs <app> --run` (on Windows) | `windows` | Windows 11 22621, x64 | [1](https://github.com/Wawona/Wawona/issues/240), [4](https://github.com/Wawona/Wawona/issues/244) |
| Android | partial: a release APK and an upload-signed App Bundle; Android TV qualified; no update stream | `bun scripts/agent-android.mjs build <app> --apk` ([the artifact](#android-release-artifact)) | `android` | API 31; 30 with `host.android.minimumOS: "11"`; 29 with `"10"` for an app whose data is Rust (the pinned Hermes bundle needs 30) | [1](https://github.com/Wawona/Wawona/issues/241), [5](https://github.com/Wawona/Wawona/issues/245) |
| visionOS | early: the iOS presenter boots on the Apple Vision Pro simulator; no device build, agent target, smoke or delivery | `bun host/apple/build.mjs --visionos --run` | none | 2.0 | [2](https://github.com/Wawona/Wawona/issues/242) |
| watchOS | snapshot complications and a live entry (`ExactWatch`, LLP 1126); a TypeScript app's entry runs on a paired simulator when it reaches no `app.ts` source; no device run | `bun host/apple/build.mjs --watchos` | none | 10 | [3](https://github.com/Wawona/Wawona/issues/243) |
| WearOS | absent: the phone APK runs unadapted; a host proposal below | | none | | [5](https://github.com/Wawona/Wawona/issues/245) |
| AsteroidOS | absent: fork goal, a Qt6/QML host | | none | | [6](https://github.com/Wawona/Wawona/issues/246) |

The Apple hosts build with Xcode 27: ExactKit calls the iOS and tvOS 27 SDKs' AVAudioSession life cycle, and
Xcode 26's Swift does not compile it.

A host is shipping when an app built for it can be driven by `scripts/agent.mjs`, smoked
by `scripts/smoke.mjs` and delivered by `scripts/deploy.mjs`. Partial lacks one of those;
early lacks more than one.

## visionOS

visionOS runs the iOS presenter (`os(visionOS)` beside `os(iOS)`, `target_os = "visionos"` beside `"ios"` and
`"tvos"`), baked as an `ios` app. UIKit on visionOS lacks some of what the iOS presenter uses, and there the
presenter does without: haptics, orientation locks, sheet detents, the status bar, keyboard accessory bars,
navigation subtitles, glass effects and glass groups (a visionOS window is glass already; a glass group refuses
with `os`), and AVKit's dynamic range preference. A visionOS window has no screen, so `UIScreen` is a stand-in that answers from the window. The
agent's accessibility snapshot does not answer on the visionOS simulator yet.

## Android release artifact

`bun scripts/agent-android.mjs build <app> --apk` runs `./gradlew :app:assembleRelease` and prints the APK's path,
`host/android/app/build/<app>/outputs/apk/release/app-release.apk`, with the build's receipt (`exact-build.json`)
beside it. The APK is minified and its `minSdkVersion` is the manifest's floor: `module-fixture` built with
`minimumOS` `"10"` records 29, and with `"11"` records 30.

`--aab` runs `./gradlew :app:bundleRelease` instead and prints
`host/android/app/build/<app>/outputs/bundle/release/app-release.aab`, the Android App Bundle Google Play
requires for new apps ([About Android App Bundles](https://developer.android.com/guide/app-bundle)). A bundle
installs on nothing; Play splits it into APKs. The build reads its identity from the environment:

- `EXACT_ANDROID_UPLOAD_KEYSTORE` names the upload keystore, with `EXACT_ANDROID_UPLOAD_STORE_PASSWORD`,
  `EXACT_ANDROID_UPLOAD_KEY_ALIAS` (default `upload`) and `EXACT_ANDROID_UPLOAD_KEY_PASSWORD` (default the store
  password). Play re-signs with the app signing key it holds
  ([Sign your app](https://developer.android.com/studio/publish/app-signing#sign_release)). Unset, the APK and
  the bundle are signed with the debug key (`CN=Android Debug`): they install by `adb` or sideload, and Play
  refuses them.
- `EXACT_BUILD_NUMBER` is the `versionCode` (default 1) and `EXACT_VERSION` the `versionName` (default `0.1.0`).
  Play refuses a `versionCode` it has seen, so each upload needs a new one.

The manifest declares the `LEANBACK_LAUNCHER` category, a banner, and touch as not required, so the same APK
installs and launches on Android TV. On the Android TV emulator (API 36, `leanback_only`) Caltrain's release
APK launched from the TV launcher's intent, D-pad Down moved focus between its buttons, Center activated the
focused one and Back returned.

## Android updates (sketch)

What a signed update stream for Android needs; nothing of it exists yet, and `scripts/deploy.mjs` publishes none
(its platforms are `web`, `ios`, `macos` and `linux`).

- **The store.** The reader opens an `exact_update::Client` before the boot, as `host/linux-update` does for
  Linux. It lives in the app's no-backup files directory
  ([`Context.getNoBackupFilesDir`](https://developer.android.com/reference/android/content/Context#getNoBackupFilesDir())):
  it is a cache of the origin, so Auto Backup must not restore it onto another install. The activity passes that
  path to `Native.start` beside the plan directory it passes today.
- **The wake.** A finished check posts to the owner thread's looper, which drains its line as Linux's
  `poll_update` does and asks for a turn through the Choreographer.
- **What a stream carries.** `deploy.mjs` gains `android`: a plan and asset update per compatibility id, signed
  with the app's update key. A changed native library is not an update; it is a new bundle through Play. Play's
  policy allows interpreted code that does not change the app's purpose
  ([Device and Network Abuse](https://support.google.com/googleplay/android-developer/answer/9888379)).
- **The receipt.** As for Windows, an update-capable Android build's receipt follows the app's store rather than
  the platform's.

## WearOS host (proposal)

Wear OS runs Android apps, so the Wear host is a variant of the Android host, not a new one. On a Wear OS 34
emulator (`wearos_small_round`, 384 × 384 round), Caltrain's unchanged release APK installed, launched and drew
its page. Someone who knows the watch would notice four things:

- The activity has no window background, so the watch face shows around the page. A watch app is black behind
  its content.
- The system reports no insets on the round display, so the host does not know the corners are cut off and the
  page's edges were clipped. The shape is
  [`Configuration.isScreenRound`](https://developer.android.com/reference/android/content/res/Configuration#isScreenRound()).
- Neither a vertical swipe nor the crown scrolled the page. The crown arrives as a `MotionEvent` from
  `InputDevice.SOURCE_ROTARY_ENCODER` with `AXIS_SCROLL`, and a Wear app scrolls its focused list with it
  ([rotary input](https://developer.android.com/training/wearables/user-input/rotary-input)).
- Frames ran slowly: Choreographer skipped 30 to 60 frames at a time on the emulator. That needs measuring on a
  watch before anyone calls it the host's cost.

The variant:

- **Packaging.** Play delivers a Wear app as its own artifact, which declares
  `android.hardware.type.watch` and the `com.google.android.wearable.standalone` metadata; the phone APK must
  declare neither ([Wear app packaging](https://developer.android.com/training/wearables/packaging)). So
  `scripts/agent-android.mjs` gains `--wear`, which builds the same `canvas` and Rust host with that manifest
  overlay and a black window background.
- **The round screen.** The host passes the shape to layout as
  [CSS Round Display](https://www.w3.org/TR/css-round-display-1/)'s `shape: round` media feature, and the
  inscribed square's margins as `env(safe-area-inset-*)`, so a page that says nothing keeps its text inside the
  circle.
- **The crown.** The host routes `AXIS_SCROLL` from the rotary encoder to the scroller that has focus, as a wheel
  is routed on the desktop hosts.
- **Back.** A swipe from the left edge did not dismiss the app on the emulator. The host registers a back
  callback, and at the app's root it must leave the gesture to the system, which dismisses a watch app
  ([swipe to dismiss](https://developer.android.com/training/wearables/views/exit)).

Tiles and complications are out of scope for the first host: like watchOS's (LLP 1126), they need a decision
of their own. The host is committed when an LLP records the variant and `--wear` builds Caltrain, launches it on
the Wear emulator, and an agent drive scrolls it with the crown.

## Windows floor

The Windows host supports Windows 11 and not Windows 10. Windows 10 left support on 2025-10-14, and its
security updates continue only for PCs enrolled in Extended Security Updates, until 2027-10-12
([Windows 10 ESU](https://www.microsoft.com/windows/extended-security-updates)). The API floor stays build
22621 (`TargetPlatformMinVersion` in `host/windows/reference/Reference.csproj`), and builds are qualified on
Windows 11 releases Microsoft still services
([Windows 11 lifecycle](https://learn.microsoft.com/lifecycle/products/windows-11-home-and-pro)). Nothing in the
host requires Windows 11: where DWM refuses Mica, `host/ink`'s composition presents an opaque frame instead.

## Windows delivery (sketch)

What phase 4's signed delivery builds; nothing of it exists yet.

- **The store.** `host/windows-update`, a sibling of `host/linux-update` above `exact-windows`, opens an
  `exact_update::Client` before the boot, as Linux's does. It lives in the per-user local application data
  folder ([`FOLDERID_LocalAppData`](https://learn.microsoft.com/windows/win32/shell/knownfolderid#FOLDERID_LocalAppData),
  `%LOCALAPPDATA%\exact\<app id>\update`): it is a cache of the origin, so it is not roamed. `EXACT_UPDATE_DIR`,
  `EXACT_UPDATE_ORIGIN` and the agent's temporary store behave as on Linux. The check runs over rustls.
- **The wake.** Linux wakes its display loop with one byte on a socketpair. The Windows display loop is winit's,
  so a finished check sends an event through the `EventLoopProxy` the host already holds, and the presenter drains
  its line as Linux's `poll_update` does.
- **What a stream carries.** `scripts/deploy.mjs` gains `windows` in its platforms: a plan and asset update per
  compatibility id, signed with the app's update key as the other native streams are. A changed executable is
  not an update; it is a new install, Authenticode-signed and packaged as MSIX so that Windows verifies and
  replaces it ([App installer](https://learn.microsoft.com/windows/msix/app-installer/app-installer-root)).
- **The receipt.** A Windows build's receipt records the bake's watches (the Contract, `app.ts` and what it
  imports) as binary inputs, since a binary-only executable carries its plan. An update-capable Windows app
  replaces its plan, so once `host/windows-update` lands that rule follows the app's store (`store.L == "0"`), as
  `scripts/app.mjs` `completeBuild` decides it today for the platform as a whole.

## Phases

| Phase | Milestone | Exit |
|---|---|---|
| 0 Charter | [24](https://github.com/Wawona/Wawona/milestone/24) | this table |
| 1 Partial-host foundations | [25](https://github.com/Wawona/Wawona/milestone/25) | tvOS driven by the agent; Windows answers `check` and refuses a stale build; Android at API 29 and 30 produces a release artifact |
| 2 visionOS presenter | [26](https://github.com/Wawona/Wawona/milestone/26) | a Contract app boots on the Vision Pro simulator from `--visionos` |
| 3 watchOS product path | [27](https://github.com/Wawona/Wawona/milestone/27) | a TypeScript app's watch entry on a paired simulator and a device |
| 4 Windows product parity | [28](https://github.com/Wawona/Wawona/milestone/28) | native controls on the Windows host; signed delivery; Windows 10 floor decided |
| 5 Android delivery, WearOS | [29](https://github.com/Wawona/Wawona/milestone/29) | store packaging and updates; Android TV qualified; a WearOS host proposal |
| 6 AsteroidOS | [30](https://github.com/Wawona/Wawona/milestone/30) | a hello Contract app on an AsteroidOS image, driven headless |
