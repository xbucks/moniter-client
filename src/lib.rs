//! ## Example
//! [Open full example with winit here 🢅](https://github.com/hirokimoto/monitor/blob/master/examples/winit/src/main.rs)

#[cfg(target_os = "windows")]
#[path = "./sys/windows/mod.rs"]
mod sys;
mod tray;
mod zip;

mod track;
mod capture;

// Public api
pub use crate::tray::icon::Icon;
pub use crate::tray::menubuilder::{MenuBuilder, MenuItem};
pub use crate::tray::trayicon::TrayIcon;
pub use crate::tray::trayiconbuilder::Error;
pub use crate::tray::trayiconbuilder::TrayIconBuilder;
pub use crate::track::{
    Button, DisplayError, Event, EventType, GrabCallback, GrabError, Key, KeyboardState,
    ListenError, SimulateError,
};
pub use crate::zip::compression::{CompressionMethod, SUPPORTED_COMPRESSION_METHODS};
pub use crate::zip::read::ZipArchive;
pub use crate::zip::types::DateTime;
pub use crate::zip::write::ZipWriter;
pub use crate::zip::result::ZipResult;
pub use crate::capture::Screen;
pub use zip::write::FileOptions;

use anyhow::{anyhow, Result};
use image::RgbaImage;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
pub use crate::macos::Keyboard;
#[cfg(target_os = "macos")]
use crate::macos::{display_size as _display_size, listen as _listen, simulate as _simulate};
#[cfg(target_os = "macos")]
use macos::*;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub use crate::linux::Keyboard;
#[cfg(target_os = "linux")]
use crate::linux::{display_size as _display_size, listen as _listen, simulate as _simulate};
#[cfg(target_os = "linux")]
use linux::*;

#[cfg(target_os = "windows")]
mod win;
#[cfg(target_os = "windows")]
pub use crate::win::Keyboard;
#[cfg(target_os = "windows")]
use crate::win::{display_size as _display_size, listen as _listen, simulate as _simulate};

#[cfg(target_os = "windows")]
use win::*;

// Each OS specific implementation must export following:
pub(crate) use crate::sys::{
    // MenuBuilder<T> -> Result<MenuSys<T>, Error>
    build_menu,

    // TrayIconBuilder<T> -> Result<Box<TrayIconSys<T>>, Error>
    build_trayicon,

    // Struct that must implement IconBase + Clone
    IconSys,

    // Struct
    MenuSys,

    // Struct that must implement TrayIconBase
    TrayIconSys,
};

/// TrayIconSys must implement this
pub(crate) trait TrayIconBase<T>
where
    T: PartialEq + Clone + 'static,
{
    fn set_icon(&mut self, icon: &Icon) -> Result<(), Error>;
    fn set_menu(&mut self, menu: &MenuBuilder<T>) -> Result<(), Error>;
    fn set_tooltip(&mut self, tooltip: &str) -> Result<(), Error>;
}

/// IconSys must implement this
pub(crate) trait IconBase {
    fn from_buffer(
        buffer: &'static [u8],
        width: Option<u32>,
        height: Option<u32>,
    ) -> Result<IconSys, Error>;
}

pub fn listen<T>(callback: T) -> Result<(), ListenError>
where
    T: FnMut(Event) + 'static,
{
    _listen(callback)
}

pub fn simulate(event_type: &EventType) -> Result<(), SimulateError> {
    _simulate(event_type)
}

pub fn display_size() -> Result<(u64, u64), DisplayError> {
    _display_size()
}

#[cfg(feature = "unstable_grab")]
#[cfg(target_os = "linux")]
pub use crate::linux::grab as _grab;
#[cfg(feature = "unstable_grab")]
#[cfg(target_os = "macos")]
pub use crate::macos::grab as _grab;
#[cfg(feature = "unstable_grab")]
#[cfg(target_os = "windows")]
pub use crate::win::grab as _grab;
#[cfg(any(feature = "unstable_grab"))]

#[cfg(any(feature = "unstable_grab"))]
pub fn grab<T>(callback: T) -> Result<(), GrabError>
where
    T: Fn(Event) -> Option<Event> + 'static,
{
    _grab(callback)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyboard_state() {
        // S
        let mut keyboard = Keyboard::new().unwrap();
        let char_s = keyboard.add(&EventType::KeyPress(Key::KeyS)).unwrap();
        assert_eq!(
            char_s,
            "s".to_string(),
            "This test should pass only on Qwerty layout !"
        );
        let n = keyboard.add(&EventType::KeyRelease(Key::KeyS));
        assert_eq!(n, None);

        // Shift + S
        keyboard.add(&EventType::KeyPress(Key::ShiftLeft));
        let char_s = keyboard.add(&EventType::KeyPress(Key::KeyS)).unwrap();
        assert_eq!(char_s, "S".to_string());
        let n = keyboard.add(&EventType::KeyRelease(Key::KeyS));
        assert_eq!(n, None);
        keyboard.add(&EventType::KeyRelease(Key::ShiftLeft));

        // Reset
        keyboard.add(&EventType::KeyPress(Key::ShiftLeft));
        keyboard.reset();
        let char_s = keyboard.add(&EventType::KeyPress(Key::KeyS)).unwrap();
        assert_eq!(char_s, "s".to_string());
        let n = keyboard.add(&EventType::KeyRelease(Key::KeyS));
        assert_eq!(n, None);
        keyboard.add(&EventType::KeyRelease(Key::ShiftLeft));

        // CapsLock
        let char_c = keyboard.add(&EventType::KeyPress(Key::KeyC)).unwrap();
        assert_eq!(char_c, "c".to_string());
        keyboard.add(&EventType::KeyPress(Key::CapsLock));
        keyboard.add(&EventType::KeyRelease(Key::CapsLock));
        let char_c = keyboard.add(&EventType::KeyPress(Key::KeyC)).unwrap();
        assert_eq!(char_c, "C".to_string());
        let n = keyboard.add(&EventType::KeyRelease(Key::KeyS));
        assert_eq!(n, None);
        keyboard.add(&EventType::KeyPress(Key::CapsLock));
        keyboard.add(&EventType::KeyRelease(Key::CapsLock));
        let char_c = keyboard.add(&EventType::KeyPress(Key::KeyC)).unwrap();
        assert_eq!(char_c, "c".to_string());
        let n = keyboard.add(&EventType::KeyRelease(Key::KeyS));
        assert_eq!(n, None);
    }
}