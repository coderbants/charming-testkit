//! Re-exports of the mouse/keys helpers (kept as a module for future
//! extensions; the actual implementations live in [crate::keys]).

pub use crate::keys::{
    mouse_click, mouse_drag, mouse_motion, mouse_release, mouse_right_click, mouse_sequence,
    MouseEvent, MOUSE_LEFT, MOUSE_MIDDLE, MOUSE_MOTION, MOUSE_RELEASE, MOUSE_RIGHT,
    MOUSE_WHEEL_DOWN, MOUSE_WHEEL_UP,
};
