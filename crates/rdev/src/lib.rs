mod rdev;
pub use crate::rdev::{Button, Event, EventType, Key, KeyboardState, ListenError};

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
use crate::macos::listen as _listen;
#[cfg(target_os = "macos")]
pub use crate::macos::{Keyboard, set_is_main_thread};

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub use crate::linux::Keyboard;
#[cfg(target_os = "linux")]
use crate::linux::listen as _listen;

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
pub use crate::windows::Keyboard;
#[cfg(target_os = "windows")]
use crate::windows::listen as _listen;

pub fn listen<T>(callback: T) -> Result<(), ListenError>
where
    T: FnMut(Event) + 'static,
{
    _listen(callback)
}
