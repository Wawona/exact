# Wawona Exact chrome (architecture charter)

Parent: [Wawona/Wawona#227](https://github.com/Wawona/Wawona/issues/227).
M0 exit: [Wawona/Wawona#228](https://github.com/Wawona/Wawona/issues/228).

## Split

| Layer | Owns | Does not own |
|---|---|---|
| Exact Contract (`apps/wawona`) | Machines list, Settings chrome, Welcome, editors shell | Wayland protocol, Smithay seat |
| Exact native hosts | Platform controls, bars, type, Present native view | Compositor policy |
| Rust `wawona-data` (M3+) | Machine profiles, prefs allowlist, launch validation | Pixel present |
| Smithay + iLand | Wayland compositor + userspace DRM present | Product chrome |

## Present-as-native-view (ABI sketch)

Exact host exposes a Present surface (`wayland-present` module, planned):

1. Contract places a Present stub (`testId="wawona-present-stub"`).
2. Host mounts `WWNIlandPresenter` / JNI / WinUI swapchain into that view.
3. Rust compositor keeps `WWNCore*` poll ABI. No UniFFI frame callbacks.

## Floors

| Host | Exact floor | Wawona product floor |
|---|---|---|
| iOS / iPadOS Exact chrome | Exact `host.ios.minimumOS` 17.0 | Wawona Mach-O still 15.0 for SwiftUI dual-run |
| macOS | 14.0 | unchanged |
| Windows | Windows 11 22621 | Exact WinUI host |
| watchOS / WearOS / AsteroidOS | Exact host matrix | M5 / M7 |

Dual-run: SwiftUI / Compose chrome stay until M8 retires them. Exact chrome is
the forward path on every Exact host.

## Nix / cargo xtask

Wawona product builds stay `cargo xtask` (Apple / Android) plus Nix slices.
Windows Exact chrome builds on the Windows guest (`bun host/windows/build.mjs
wawona`) until Nix Windows cross lands (M6).
