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
| tvOS | partial: builds and runs on the iOS presenter; no smoke | `bun host/apple/build.mjs --tvos --run` | `tvos` | 17.0 (the iOS floor) | [1](https://github.com/Wawona/Wawona/issues/239) |
| Windows | partial: runs; no delivery | `bun host/windows/build.mjs <app> --run` (on Windows) | `windows` | Windows 11 22621, x64 | [1](https://github.com/Wawona/Wawona/issues/240), [4](https://github.com/Wawona/Wawona/issues/244) |
| Android | early: no store delivery | `bun exact.mjs android` (a release APK under `target/android/`) | `android` | API 31; 30 with `host.android.minimumOS: "11"`; 29 with `"10"` for an app whose data is Rust (the pinned Hermes bundle needs 30) | [1](https://github.com/Wawona/Wawona/issues/241), [5](https://github.com/Wawona/Wawona/issues/245) |
| visionOS | absent: declared in `Package.swift` only | | none | | [2](https://github.com/Wawona/Wawona/issues/242) |
| watchOS | snapshot only (`ExactWatch`, LLP 1126); no TypeScript apps | `bun host/apple/build.mjs --watchos` | none | 10 | [3](https://github.com/Wawona/Wawona/issues/243) |
| WearOS | absent: fork goal | | none | | [5](https://github.com/Wawona/Wawona/issues/245) |
| AsteroidOS | absent: fork goal, a Qt6/QML host | | none | | [6](https://github.com/Wawona/Wawona/issues/246) |

The Apple hosts build with Xcode 27: ExactKit calls the iOS and tvOS 27 SDKs' AVAudioSession life cycle, and
Xcode 26's Swift does not compile it.

A host is shipping when an app built for it can be driven by `scripts/agent.mjs`, smoked
by `scripts/smoke.mjs` and delivered by `scripts/deploy.mjs`. Partial lacks one of those;
early lacks more than one.

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
