extern crate winapi;

mod common;
mod display;
mod win32;
#[cfg(feature = "unstable_grab")]
mod grab;
mod keyboard;
mod keycodes;
mod listen;
mod simulate;

pub use crate::win::display::display_size;
#[cfg(feature = "unstable_grab")]
pub use crate::win::grab::grab;
pub use crate::win::keyboard::Keyboard;
pub use crate::win::listen::listen;
pub use crate::win::simulate::simulate;
