//! iOS display-synchronized redraw delivery for the existing UIKit view.

use std::cell::RefCell;
use std::collections::HashSet;

use objc2::rc::{Retained, Weak};
use objc2::{declare_class, msg_send_id, mutability, sel, ClassType, DeclaredClass};
use objc2_foundation::{MainThreadMarker, NSObject, NSRunLoop, NSRunLoopCommonModes};
use objc2_quartz_core::{CADisplayLink, CAFrameRateRange};

use super::app_state::{self, EventWrapper};
use super::view::WinitView;
use crate::event::{Event, TouchPhase, WindowEvent};
use crate::window::WindowId as RootWindowId;

/// Owns the display link and touch activity for a single UIKit view.
#[derive(Default)]
pub(super) struct DisplayLinkDriver {
    link: RefCell<Option<Retained<CADisplayLink>>>,
    active_touches: RefCell<HashSet<u64>>,
}

impl DisplayLinkDriver {
    pub(super) fn update_touch(&self, id: u64, phase: TouchPhase) {
        match phase {
            TouchPhase::Started => {
                self.active_touches.borrow_mut().insert(id);
            },
            TouchPhase::Ended | TouchPhase::Cancelled => {
                self.active_touches.borrow_mut().remove(&id);
            },
            TouchPhase::Moved => {},
        }
    }

    pub(super) fn has_active_touches(&self) -> bool {
        !self.active_touches.borrow().is_empty()
    }

    pub(super) fn request(&self, view: &WinitView) {
        let mtm = MainThreadMarker::from(view);
        let mut slot = self.link.borrow_mut();
        if slot.is_none() {
            let Some(window) = view.window() else {
                view.setNeedsDisplay();
                return;
            };
            let target = mtm.alloc().set_ivars(RedrawTargetState { view: Weak::new(view) });
            let target: Retained<RedrawTarget> = unsafe { msg_send_id![super(target), init] };
            let link = unsafe {
                CADisplayLink::displayLinkWithTarget_selector(&target, sel!(displayLinkFired:))
            };
            let maximum = window.screen().maximumFramesPerSecond() as f32;
            unsafe {
                link.setPreferredFrameRateRange(CAFrameRateRange {
                    minimum: maximum.min(10.0),
                    maximum,
                    preferred: maximum,
                });
                link.setPaused(true);
                link.addToRunLoop_forMode(&NSRunLoop::mainRunLoop(), NSRunLoopCommonModes);
            }
            *slot = Some(link);
        }
        if let Some(link) = slot.as_ref() {
            unsafe { link.setPaused(false) };
        }
    }
}

impl Drop for DisplayLinkDriver {
    fn drop(&mut self) {
        if let Some(link) = self.link.get_mut().take() {
            unsafe { link.invalidate() };
        }
    }
}

struct RedrawTargetState {
    view: Weak<WinitView>,
}

declare_class!(
    struct RedrawTarget;

    unsafe impl ClassType for RedrawTarget {
        type Super = NSObject;
        type Mutability = mutability::MainThreadOnly;
        const NAME: &'static str = "WinitIOSRedrawTarget";
    }

    impl DeclaredClass for RedrawTarget {
        type Ivars = RedrawTargetState;
    }

    unsafe impl RedrawTarget {
        #[method(displayLinkFired:)]
        fn display_link_fired(&self, link: &CADisplayLink) {
            let Some(view) = self.ivars().view.load() else {
                unsafe { link.invalidate() };
                return;
            };
            if !view.display_link().has_active_touches() {
                unsafe { link.setPaused(true) };
            }
            let Some(window) = view.window() else { return };
            let mtm = MainThreadMarker::from(&*view);
            app_state::handle_nonuser_event(
                mtm,
                EventWrapper::StaticEvent(Event::WindowEvent {
                    window_id: RootWindowId(window.id()),
                    event: WindowEvent::RedrawRequested,
                }),
            );
        }
    }
);
