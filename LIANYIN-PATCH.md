# Lianyin iOS adaptations

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
drop. It is an iOS-only adapter; other platform implementations are unchanged.
Remove each adaptation when a maintained upstream release provides equivalent
behavior and the application's representative native paths pass.
