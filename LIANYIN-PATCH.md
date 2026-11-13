# Lianyin platform adaptations

This branch is based on the upstream `v0.30.13` commit
`e9809ef54b18499bb4f2cac945719ecc2a61061b` and retains the `winit` package
version `0.30.13`. The consuming application pins the Git revision.

- The iOS `UITextInput` adapter is based on upstream
  [PR #4592](https://github.com/rust-windowing/winit/pull/4592), head
  `e49d771a53e5fd6e060aa9f96e91a69a2cc9a64c`. It translates UIKit UTF-16
  positions to winit UTF-8 preedit offsets, preserves composition cancellation,
  and sends committed text once to Bevy's existing editor.
- The iOS display-link adapter preserves the existing `UIView` and input path.
  A redraw request wakes one display-synchronized callback; active touches keep
  callbacks running until the last touch ends or is cancelled. An idle callback
  pauses the link. Bevy retains its reactive update and frame-limit policies.
  The app must set `CADisableMinimumFrameDurationOnPhone` to request high refresh
  rates; the system still chooses the actual rate.
- `dpi` resolves from crates.io as it does in the published `winit` package,
  rather than from this repository's older workspace copy. This preserves the
  existing public DPI types when the application switches from a registry package
  to a Git dependency.

The display link is owned per view, uses a weak target, and is invalidated on
drop. This adapter is specific to iOS.
Remove each adaptation when a maintained upstream release provides equivalent
behavior and the application's representative native paths pass.

## Android requested-redraw pacing

The optional `android-frame-pacing` feature adds
`WindowExtAndroid::set_native_frame_pacing(bool)` for Android API 24 and later.
It is disabled until the consuming application opts in. Requested redraws are
coalesced into a one-shot `AChoreographer` callback on the existing android_main
Looper. Idle callbacks do not rearm themselves; no extra thread or animation
state is introduced. The callback owns an Arc independently of event-loop
lifetime, and input/lifecycle handling remains in the existing loop.

Applications should disable native pacing while another limiter owns frame
pacing. Lianyin uses native callbacks for its adaptive setting, the existing
framepace limiter for explicit caps, and Bevy's main-frame time for UI motion.
The legacy callback timestamp is unused, retaining API 24 compatibility.

This pins upstream [android-activity PR #250](https://github.com/rust-mobile/android-activity/pull/250)
(`be04f9d82820b423091d3f5ca24d310db023335d`), which forwards normal
`ALOOPER_POLL_CALLBACK` completion as `PollEvent::Callback`. The Android event
loop handles it alongside Wake before inspecting pending redraws. The original
0.6.1 glue incorrectly logs that normal return value as an error and ignores it.
Consumers must patch registry android-activity to the same Git source so Winit
and the host share one AndroidApp type. See the [NDK Choreographer contract](https://developer.android.com/ndk/reference/group/choreographer)
and [Looper return values](https://developer.android.com/ndk/reference/group/looper).

Android device verification and candidate identities are tracked by the
consuming application's UI performance plan. Remove this capability when the
maintained Winit Android backend supplies equivalent requested-redraw pacing.
