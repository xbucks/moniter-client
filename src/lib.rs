//! ## Example
//! [Open full example with winit here 🢅](https://github.com/hirokimoto/monitor/blob/master/examples/winit/src/main.rs)

#[cfg(target_os = "windows")]
#[path = "./sys/windows/mod.rs"]
mod sys;

mod icon;
mod menubuilder;
mod trayicon;
mod trayiconbuilder;
mod trayiconsender;
mod kmdev;
mod image_utils;
mod zip;

// Public api
pub use crate::icon::Icon;
pub use crate::menubuilder::{MenuBuilder, MenuItem};
pub use crate::trayicon::TrayIcon;
pub use crate::trayiconbuilder::Error;
pub use crate::trayiconbuilder::TrayIconBuilder;
pub use crate::kmdev::{
    Button, DisplayError, Event, EventType, GrabCallback, GrabError, Key, KeyboardState,
    ListenError, SimulateError,
};
pub use crate::zip::compression::{CompressionMethod, SUPPORTED_COMPRESSION_METHODS};
pub use crate::zip::read::ZipArchive;
pub use crate::zip::types::DateTime;
pub use crate::zip::write::ZipWriter;
pub use crate::zip::result::ZipResult;
pub use display_info::DisplayInfo;
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


#[derive(Debug, Clone, Copy)]
pub struct Screen {
    pub display_info: DisplayInfo,
}

impl Screen {
    pub fn new(display_info: &DisplayInfo) -> Self {
        Screen {
            display_info: *display_info,
        }
    }

    pub fn all() -> Result<Vec<Screen>> {
        let screens = DisplayInfo::all()?.iter().map(Screen::new).collect();
        Ok(screens)
    }

    pub fn from_point(x: i32, y: i32) -> Result<Screen> {
        let display_info = DisplayInfo::from_point(x, y)?;
        Ok(Screen::new(&display_info))
    }

    pub fn capture(&self) -> Result<RgbaImage> {
        capture_screen(&self.display_info)
    }

    /**
     * 截取指定区域
     * 区域x,y为相对于当前屏幕的x,y坐标
     */
    pub fn capture_area(&self, x: i32, y: i32, width: u32, height: u32) -> Result<RgbaImage> {
        let display_info = self.display_info;
        let screen_x2 = display_info.x + display_info.width as i32;
        let screen_y2 = display_info.y + display_info.height as i32;

        let mut x1 = x + display_info.x;
        let mut y1 = y + display_info.y;
        let mut x2 = x1 + width as i32;
        let mut y2 = y1 + height as i32;

        // x y 必须在屏幕范围内
        if x1 < display_info.x {
            x1 = display_info.x;
        } else if x1 > screen_x2 {
            x1 = screen_x2
        }

        if y1 < display_info.y {
            y1 = display_info.y;
        } else if y1 > screen_y2 {
            y1 = screen_y2;
        }

        if x2 > screen_x2 {
            x2 = screen_x2;
        }

        if y2 > screen_y2 {
            y2 = screen_y2;
        }

        if x1 >= x2 || y1 >= y2 {
            return Err(anyhow!("Area size is invalid"));
        }

        capture_screen_area(
            &display_info,
            x1 - display_info.x,
            y1 - display_info.y,
            (x2 - x1) as u32,
            (y2 - y1) as u32,
        )
    }
}