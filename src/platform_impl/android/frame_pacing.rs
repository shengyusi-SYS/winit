//! On-demand native display pacing. The application chooses when to enable it.
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub(super) struct State {
    enabled: AtomicBool,
    pending: AtomicBool,
    ready: AtomicBool,
}
impl State {
    pub(super) fn set_enabled(&self, enabled: bool) -> bool {
        let changed = self.enabled.swap(enabled, Ordering::AcqRel) != enabled;
        if changed {
            self.ready.store(false, Ordering::Release);
        }
        changed
    }
}

pub(super) struct FramePacing {
    choreographer: *mut ndk_sys::AChoreographer,
    pub(super) state: Arc<State>,
}
impl FramePacing {
    pub(super) fn new() -> Option<Self> {
        // Called on android_main after android-activity prepares its Looper.
        let choreographer = unsafe { ndk_sys::AChoreographer_getInstance() };
        if choreographer.is_null() {
            return None;
        }
        Some(Self {
            choreographer,
            state: Arc::new(State {
                enabled: AtomicBool::new(false),
                pending: AtomicBool::new(false),
                ready: AtomicBool::new(false),
            }),
        })
    }
    pub(super) fn ready(&self) -> bool {
        !self.state.enabled.load(Ordering::Acquire) || self.state.ready.load(Ordering::Acquire)
    }
    pub(super) fn redraw_due(&self) -> bool {
        if !self.state.enabled.load(Ordering::Acquire) {
            return true;
        }
        if self.state.ready.swap(false, Ordering::AcqRel) {
            return true;
        }
        if !self.state.pending.swap(true, Ordering::AcqRel) {
            // The one-shot callback owns this Arc independently of the event loop.
            // Only outstanding redraws schedule callbacks; idle does not rearm.
            let data = Arc::into_raw(self.state.clone()).cast_mut().cast();
            // The legacy callback supports API 24; its 32-bit timestamp is unused.
            unsafe {
                ndk_sys::AChoreographer_postFrameCallback(self.choreographer, Some(frame), data);
            }
        }
        false
    }
}
unsafe extern "C" fn frame(_: std::os::raw::c_long, data: *mut std::ffi::c_void) {
    // SAFETY: exactly one callback consumes the Arc passed by redraw_due.
    let state = unsafe { Arc::from_raw(data.cast::<State>()) };
    state.pending.store(false, Ordering::Release);
    state.ready.store(state.enabled.load(Ordering::Acquire), Ordering::Release);
    // android-activity forwards processed Looper callbacks as PollEvent::Callback.
}
