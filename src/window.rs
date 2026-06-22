#[cfg(target_os = "linux")]
use anyhow::{Context, Result};
use iced::window::{self, Level, Position};
use iced::{Point, Size};

#[derive(Debug)]
pub struct Geometry {
    pub screen_margin: f32,
}

impl Default for Geometry {
    fn default() -> Self {
        Self {
            screen_margin: 40.0,
        }
    }
}

/// Returns the window settings.
pub fn settings(size: Size, geometry: &Geometry) -> window::Settings {
    let settings = window::Settings {
        size,
        position: Position::Specific(Point::new(geometry.screen_margin, geometry.screen_margin)),
        transparent: true,
        decorations: false,
        level: Level::AlwaysOnTop,
        resizable: false,
        ..Default::default()
    };

    #[cfg(target_os = "windows")]
    let settings = {
        let mut settings = settings;
        settings.platform_specific.skip_taskbar = true;
        settings
    };

    settings
}

/// Returns the window size and position.
pub fn placement(size: Size, monitor_size: Size, geometry: &Geometry) -> (Size, Point) {
    let max_width = (monitor_size.width - geometry.screen_margin * 2.0).max(1.0);
    let max_height = (monitor_size.height - geometry.screen_margin * 2.0).max(1.0);

    let size = Size::new(size.width.min(max_width), size.height.min(max_height));
    let position = Point::new(
        geometry.screen_margin,
        (monitor_size.height - size.height - geometry.screen_margin).max(geometry.screen_margin),
    );

    (size, position)
}

#[cfg(target_os = "linux")]
pub fn configure_x11_window<Message: Send + 'static>(id: window::Id) -> iced::Task<Message> {
    use iced::window::raw_window_handle::RawWindowHandle;

    iced::window::run(id, |window| {
        let Ok(handle) = window.window_handle() else {
            return;
        };

        let xwindow = match handle.as_raw() {
            RawWindowHandle::Xlib(xlib_handle) => Some(xlib_handle.window as u32),
            RawWindowHandle::Xcb(xcb_handle) => Some(xcb_handle.window.get()),
            _ => None,
        };

        if let Some(xwindow) = xwindow
            && let Err(e) = set_x11_properties(xwindow) {
                log::warn!("{e:#}");
            }
    })
    .discard()
}

#[cfg(target_os = "linux")]
fn set_x11_properties(xwindow: u32) -> Result<()> {
    use x11rb::connection::Connection;
    use x11rb::protocol::xproto;
    use x11rb::protocol::xproto::ConnectionExt as _;
    use x11rb::wrapper::ConnectionExt as _;

    let (conn, screen_num) = x11rb::connect(None).context("failed to connect to x11")?;
    let screen = &conn.setup().roots[screen_num];

    x11rb::atom_manager! {
        pub Atoms: AtomsCookie {
            _NET_WM_STATE,
            _NET_WM_STATE_SKIP_TASKBAR,
            _NET_WM_STATE_SKIP_PAGER,
            _NET_WM_WINDOW_TYPE,
            _NET_WM_WINDOW_TYPE_UTILITY,
            _NET_WM_STATE_ABOVE,
        }
    }

    let atoms = Atoms::new(&conn)
        .context("failed to create x11 atom cookie")?
        .reply()
        .context("failed to resolve x11 atoms")?;

    conn.change_property32(
        xproto::PropMode::REPLACE,
        xwindow,
        atoms._NET_WM_WINDOW_TYPE,
        xproto::AtomEnum::ATOM,
        &[atoms._NET_WM_WINDOW_TYPE_UTILITY],
    )
    .context("failed to set x11 window type")?;

    let event_skip = xproto::ClientMessageEvent {
        response_type: xproto::CLIENT_MESSAGE_EVENT,
        format: 32,
        sequence: 0,
        window: xwindow,
        type_: atoms._NET_WM_STATE,
        data: xproto::ClientMessageData::from([
            1,
            atoms._NET_WM_STATE_SKIP_TASKBAR,
            atoms._NET_WM_STATE_SKIP_PAGER,
            1,
            0,
        ]),
    };

    conn.send_event(
        false,
        screen.root,
        xproto::EventMask::SUBSTRUCTURE_REDIRECT | xproto::EventMask::SUBSTRUCTURE_NOTIFY,
        event_skip,
    )
    .context("failed to send x11 skip-taskbar state")?;

    let event_above = xproto::ClientMessageEvent {
        response_type: xproto::CLIENT_MESSAGE_EVENT,
        format: 32,
        sequence: 0,
        window: xwindow,
        type_: atoms._NET_WM_STATE,
        data: xproto::ClientMessageData::from([1, atoms._NET_WM_STATE_ABOVE, 0, 1, 0]),
    };

    conn.send_event(
        false,
        screen.root,
        xproto::EventMask::SUBSTRUCTURE_REDIRECT | xproto::EventMask::SUBSTRUCTURE_NOTIFY,
        event_above,
    )
    .context("failed to send x11 always-on-top state")?;

    conn.flush()
        .context("failed to flush x11 window property changes")?;
    Ok(())
}
