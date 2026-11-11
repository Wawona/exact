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
| Android | early: a debug-signed release APK; no AAB, no store delivery | `bun scripts/agent-android.mjs build <app> --apk` ([the artifact](#android-release-artifact)) | `android` | API 31; 30 with `host.android.minimumOS: "11"`; 29 with `"10"` for an app whose data is Rust (the pinned Hermes bundle needs 30) | [1](https://github.com/Wawona/Wawona/issues/241), [5](https://github.com/Wawona/Wawona/issues/245) |
| visionOS | absent: declared in `Package.swift` only | | none | | [2](https://github.com/Wawona/Wawona/issues/242) |
| watchOS | snapshot only (`ExactWatch`, LLP 1126); no TypeScript apps | `bun host/apple/build.mjs --watchos` | none | 10 | [3](https://github.com/Wawona/Wawona/issues/243) |
| WearOS | absent: fork goal | | none | | [5](https://github.com/Wawona/Wawona/issues/245) |
| AsteroidOS | absent: fork goal, a Qt6/QML host | | none | | [6](https://github.com/Wawona/Wawona/issues/246) |

The Apple hosts build with Xcode 27: ExactKit calls the iOS and tvOS 27 SDKs' AVAudioSession life cycle, and
Xcode 26's Swift does not compile it.

A host is shipping when an app built for it can be driven by `scripts/agent.mjs`, smoked
by `scripts/smoke.mjs` and delivered by `scripts/deploy.mjs`. Partial lacks one of those;
early lacks more than one.

## Android release artifact

`bun scripts/agent-android.mjs build <app> --apk` runs `./gradlew :app:assembleRelease` and prints the APK's path,
`host/android/app/build/<app>/outputs/apk/release/app-release.apk`, with the build's receipt (`exact-build.json`)
beside it. The APK is minified and its `minSdkVersion` is the manifest's floor: `module-fixture` built with
`minimumOS` `"10"` records 29, and with `"11"` records 30. Two things keep it from being a store artifact:

- It is signed with the debug key (`signingConfigs.debug`, certificate `CN=Android Debug`), so it installs by
  `adb` or sideload but cannot be published or updated in place under a release identity.
- There is no Android App Bundle. Google Play requires one for new apps
  ([About Android App Bundles](https://developer.android.com/guide/app-bundle)); `:app:bundleRelease` with an
  upload key is phase 5's work ([Wawona/Wawona#245](https://github.com/Wawona/Wawona/issues/245)).

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
