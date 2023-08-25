extern crate winapi;

mod common;
mod display;
#[cfg(feature = "unstable_grab")]
mod grab;
mod keyboard;
mod keycodes;
mod listen;
mod simulate;
mod win32;

pub use crate::win::display::display_size;
pub use crate::win::win32::capture_screen;
pub use crate::win::win32::capture_screen_area;
#[cfg(feature = "unstable_grab")]
pub use crate::win::grab::grab;
pub use crate::win::keyboard::Keyboard;
pub use crate::win::listen::listen;
pub use crate::win::simulate::simulate;
