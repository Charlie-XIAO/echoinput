mod common;
mod wayland;
mod x11;

pub enum Keyboard {
    X11(x11::Keyboard),
    Wayland(wayland::Keyboard),
}

impl Keyboard {
    pub fn new() -> Option<Self> {
        if is_wayland() {
            wayland::Keyboard::new().map(Keyboard::Wayland)
        } else {
            x11::Keyboard::new().map(Keyboard::X11)
        }
    }
}

impl crate::rdev::KeyboardState for Keyboard {
    fn add(&mut self, event_type: &crate::rdev::EventType) -> Option<String> {
        match self {
            Keyboard::X11(kb) => kb.add(event_type),
            Keyboard::Wayland(kb) => kb.add(event_type),
        }
    }

    fn reset(&mut self) {
        match self {
            Keyboard::X11(kb) => kb.reset(),
            Keyboard::Wayland(kb) => kb.reset(),
        }
    }
}

pub fn listen<T>(callback: T) -> Result<(), crate::rdev::ListenError>
where
    T: FnMut(crate::rdev::Event) + 'static,
{
    if is_wayland() {
        wayland::listen(callback)
    } else {
        x11::listen(callback)
    }
}

fn is_wayland() -> bool {
    std::env::var("WAYLAND_DISPLAY")
        .ok()
        .filter(|var| !var.is_empty())
        .or_else(|| std::env::var("WAYLAND_SOCKET").ok())
        .filter(|var| !var.is_empty())
        .is_some()
}
