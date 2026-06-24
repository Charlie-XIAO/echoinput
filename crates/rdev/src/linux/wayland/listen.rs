use std::fs::{File, OpenOptions};
use std::os::unix::fs::OpenOptionsExt;
use std::os::unix::io::{AsRawFd, OwnedFd};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::SystemTime;

use input::event::PointerEvent;
use input::event::keyboard::{KeyState, KeyboardEventTrait};
use input::event::pointer::{Axis, ButtonState, PointerScrollEvent};
use input::{Event as LibEvent, Libinput, LibinputInterface};
use libc::{O_RDONLY, O_RDWR, O_WRONLY, POLLIN, poll, pollfd};

use super::keyboard::Keyboard;
use super::keycodes::key_from_code;
use crate::linux::common::{handle_numpad_fallback, is_led_on};
use crate::rdev::{Event, KeyboardState, ListenError};
use crate::{Button, EventType};

pub static INITIAL_CAPSLOCK: AtomicBool = AtomicBool::new(false);
pub static INITIAL_NUMLOCK: AtomicBool = AtomicBool::new(false);

struct Interface;

fn convert_type(libevent: LibEvent) -> Option<EventType> {
    match libevent {
        LibEvent::Keyboard(key) => {
            let k = key_from_code(key.key());
            let state: KeyState = key.key_state();
            match state {
                KeyState::Pressed => Some(EventType::KeyPress(k)),
                KeyState::Released => Some(EventType::KeyRelease(k)),
            }
        },
        LibEvent::Pointer(PointerEvent::Button(btn)) => {
            let rdev_btn = match btn.button() {
                272 => Some(Button::Left),
                273 => Some(Button::Right),
                274 => Some(Button::Middle),
                _ => None,
            };
            if let Some(rdev_btn) = rdev_btn {
                let state: ButtonState = btn.button_state();
                match state {
                    ButtonState::Pressed => Some(EventType::ButtonPress(rdev_btn)),
                    ButtonState::Released => Some(EventType::ButtonRelease(rdev_btn)),
                }
            } else {
                None
            }
        },
        LibEvent::Pointer(PointerEvent::Motion(btn)) => Some(EventType::MouseMove {
            // TODO Convert to absolute X, Y
            x: btn.dx_unaccelerated(),
            y: btn.dy_unaccelerated(),
        }),
        LibEvent::Pointer(PointerEvent::MotionAbsolute(btn)) => Some(EventType::MouseMove {
            x: btn.absolute_x(),
            y: btn.absolute_y(),
        }),
        LibEvent::Pointer(PointerEvent::ScrollWheel(btn)) => {
            let delta_x = if btn.has_axis(Axis::Horizontal) {
                -(btn.scroll_value_v120(Axis::Horizontal) / 120.0) as i64
            } else {
                0
            };
            let delta_y = if btn.has_axis(Axis::Vertical) {
                -(btn.scroll_value_v120(Axis::Vertical) / 120.0) as i64
            } else {
                0
            };
            Some(EventType::Wheel { delta_x, delta_y })
        },
        _ => None,
    }
}

fn convert(keyboard: &mut Keyboard, libevent: LibEvent) -> Option<Event> {
    let mut event_type = convert_type(libevent)?;
    let name = keyboard.add(&event_type);
    if name.is_none() {
        handle_numpad_fallback(&mut event_type);
    }

    Some(Event {
        time: SystemTime::now(),
        name,
        event_type,
    })
}

impl LibinputInterface for Interface {
    fn open_restricted(&mut self, path: &Path, flags: i32) -> Result<OwnedFd, i32> {
        #[allow(clippy::bad_bit_mask)]
        OpenOptions::new()
            .custom_flags(flags)
            .read((flags & O_RDONLY != 0) | (flags & O_RDWR != 0))
            .write((flags & O_WRONLY != 0) | (flags & O_RDWR != 0))
            .open(path)
            .map(|file| file.into())
            .map_err(|err| err.raw_os_error().unwrap())
    }
    fn close_restricted(&mut self, fd: OwnedFd) {
        drop(File::from(fd));
    }
}

pub fn listen<T>(mut callback: T) -> Result<(), ListenError>
where
    T: FnMut(Event) + 'static,
{
    if is_led_on("::capslock") {
        INITIAL_CAPSLOCK.store(true, Ordering::SeqCst);
    }
    if is_led_on("::numlock") {
        INITIAL_NUMLOCK.store(true, Ordering::SeqCst);
    }

    let mut input = Libinput::new_with_udev(Interface);
    input.udev_assign_seat("seat0").unwrap();
    let mut keyboard = Keyboard::new().ok_or(ListenError::KeyboardError)?;
    let fd = input.as_raw_fd();
    let mut fds = [pollfd {
        fd,
        events: POLLIN,
        revents: 0,
    }];

    loop {
        let ret = unsafe { poll(fds.as_mut_ptr(), 1, 10) };
        if ret < 0 {
            let err = std::io::Error::last_os_error();
            if err.kind() == std::io::ErrorKind::Interrupted {
                continue;
            }
            return Err(ListenError::KeyboardError);
        }

        input.dispatch().unwrap();
        for libevent in &mut input {
            if let Some(event) = convert(&mut keyboard, libevent) {
                callback(event);
            }
        }
    }
}
