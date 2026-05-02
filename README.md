# Apex Footwork

![Apex Footwork screenshot](docs/apex_footwork.png)

[Download](https://github.com/timofei-durakov/apex-footwork/releases/tag/v0.0.2) | [apexfootwork.com](https://apexfootwork.com) 


Apex Footwork is a native Windows utility for mapping a pedal/controller device to throttle, brake, and steering inputs, then monitoring those inputs in a lightweight on-screen overlay.

The app walks through device selection, throttle detection, brake detection, steering detection, and then shows live input values with pedal and steering history graphs.

## Important: unsigned application

This project is not code-signed yet.

Windows SmartScreen, antivirus software, or browser download protection may warn that the app or installer is from an unknown publisher. That is expected for the current builds. Treat releases as unsigned development builds until a signing certificate and release signing process are added.

## Features

- Detects joystick/HID devices through the Windows multimedia joystick API.
- Captures throttle, brake, and steering axes automatically from controller movement.
- Supports a recommended driver-range mode and an advanced custom calibration mode.
- Saves the selected device and bindings to a local profile.
- Restores the saved profile on startup when the device is connected.
- Provides a movable overlay with live throttle/brake bars, a pedal history graph, and an optional vertical steering trace.
- Includes overlay controls for steering graph visibility and sensitive steering display.
- Shows coaching alerts for pedal overlap, coasting, throttle application with steering lock, abrupt brake release, steering oscillation, and excessive steering lock.
- Embeds the project icon into the app binary, installer, uninstaller, and Start Menu shortcuts.

## Usage

1. Connect the pedal/controller device.
2. Launch Apex Footwork.
3. Select the controller that owns the pedals and steering input.
4. Click `Use device`.
5. Release all pedals, then click `Capture Throttle`.
6. Press and release the throttle pedal.
7. Repeat the capture flow for the brake pedal.
8. Center the steering input, then click `Capture Steering`.
9. In normal mode, turn steering right once and release it. The full driver range is used for steering.
10. Click `Start` to open the overlay.

Advanced calibration can be enabled before each capture step. For pedals, advanced calibration saves the custom 0-100% travel captured during setup. For steering, advanced calibration captures center, full left, and full right.

Overlay controls:

- `Steering graph`: show or hide the vertical steering history graph.
- `Sensitive steering`: use a log-style display scale that makes smaller steering movements easier to see. This changes only the graph display, not the saved binding or raw input mapping.
- `Opacity`: adjust graph opacity.

## Overlay alerts

Overlay alerts are lightweight coaching hints based only on throttle, brake, and steering input. They do not use game telemetry such as speed, gear, ABS, TC, tire slip, or car state, so treat them as technique prompts rather than absolute driving truth.

The default alert sensitivity is `balanced`. Internally the alert engine also supports `quiet` and `sensitive` presets. Alert chips appear over the pedal history graph, with at most two visible at once.

`Pedal overlap`

- Chip: `Throttle + brake`
- Severity: warning
- Balanced trigger: throttle > 12% and brake > 12% for more than 160 ms.
- Quiet trigger: throttle > 16% and brake > 16% for more than 240 ms.
- Sensitive trigger: throttle > 8% and brake > 8% for more than 96 ms.
- Useful for spotting unwanted throttle/brake overlap.

`Coasting`

- Chip: `Coasting`
- Severity: notice
- Balanced trigger: throttle < 4%, brake < 4%, and steering > 12% for more than 350 ms.
- Quiet trigger: throttle < 3%, brake < 3%, and steering > 16% for more than 500 ms.
- Sensitive trigger: throttle < 6%, brake < 6%, and steering > 10% for more than 240 ms.
- Useful for spotting cornering phases with no longitudinal load from either pedal.

`Throttle with lock`

- Chip: `Unwind first`
- Severity: warning
- Balanced trigger: steering > 45%, current throttle >= 28%, and throttle rises by at least 22% over roughly 160 ms.
- Quiet trigger: steering > 55%, current throttle >= 34%, and throttle rises by at least 30% over roughly 160 ms.
- Sensitive trigger: steering > 35%, current throttle >= 22%, and throttle rises by at least 16% over roughly 160 ms.
- Useful for spotting aggressive throttle application before unwinding the steering on corner exit.

`Brake release snap`

- Chip: `Ease release`
- Severity: warning
- Balanced trigger: steering > 20%, brake was >= 35% roughly 160 ms ago, and brake drops by at least 28%.
- Quiet trigger: steering > 28%, brake was >= 45% roughly 160 ms ago, and brake drops by at least 36%.
- Sensitive trigger: steering > 15%, brake was >= 28% roughly 160 ms ago, and brake drops by at least 20%.
- Useful for spotting abrupt brake release while the car is still turned in.

`Steering saw`

- Chip: `Sawing wheel`
- Severity: notice
- Balanced trigger: at least 3 steering direction changes in roughly 640 ms, each counted movement >= 8%, with at least 16% total steering range.
- Quiet trigger: at least 4 direction changes, each counted movement >= 10%, with at least 24% total steering range.
- Sensitive trigger: at least 2 direction changes, each counted movement >= 6%, with at least 12% total steering range.
- Useful for spotting rapid steering corrections instead of one stable steering input.

`Steering saturated`

- Chip: `Too much lock`
- Severity: warning
- Balanced trigger: steering > 92% and throttle or brake > 8% for more than 350 ms.
- Quiet trigger: steering > 96% and throttle or brake > 12% for more than 500 ms.
- Sensitive trigger: steering > 88% and throttle or brake > 6% for more than 240 ms.
- Useful for spotting excessive steering lock, sustained understeer-like input, or possible steering calibration issues.

Useful shortcuts:

- `Enter`: confirm the current setup step.
- `Ctrl+Shift+O`: start or stop the overlay when configured.
- `Ctrl+Shift+R`: return to configuration.
- `Alt+F4`: exit.

## Saved profile

The profile is stored per user:

```text
%APPDATA%\ApexFootwork\profile.txt
```

If `%APPDATA%` is unavailable, the app falls back to `%LOCALAPPDATA%`, then the current working directory.

The profile stores the selected device, throttle/brake/steering bindings, calibration data, overlay display settings, and overlay alert settings.

## Build requirements

- Windows
- Rust toolchain with Cargo
- Windows SDK resource compiler, `rc.exe`
- NSIS with `makensis.exe` available in `PATH` for installer builds

MSVC Windows builds link the Visual C++ runtime statically via `.cargo/config.toml`,
so release builds do not require users to install the Microsoft Visual C++
Redistributable separately.

## Build the app

```powershell
cargo build --release
```

The release binary is written to:

```text
target\release\apex_footwork.exe
```

## Build the installer

```powershell
powershell -ExecutionPolicy Bypass -File scripts\build-installer.ps1
```

The installer is written to:

```text
dist\ApexFootwork-<version>-setup.exe
```

The installer currently installs per user into:

```text
%LOCALAPPDATA%\Programs\ApexFootwork
```

## Project layout

```text
apex-footwork.ico              Application and installer icon
build.rs                       Windows resource embedding for the app binary
src\main.rs                    Win32 UI, overlay, device polling, app entry point
src\alerts.rs                  Overlay alert detection engine and alert presets
src\wizard.rs                  Device selection and pedal capture workflow
src\profile.rs                 Saved profile serialization and loading
installer\apex-footwork.nsi    NSIS installer script
scripts\build-installer.ps1    Release and installer build script
```

## Release notes

Current builds are development builds and are not signed.
