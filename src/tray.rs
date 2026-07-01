use anyhow::{Context, Result};
use trayinit::{Icon, Menu, MenuNode, Tray, TrayEvent, TrayState};

fn checker_icon_rgba(size: usize) -> Vec<u8> {
    let mut rgba = Vec::with_capacity(size * size * 4);
    for y in 0..size {
        for x in 0..size {
            let light = ((x / 8) + (y / 8)) % 2 == 0;
            let (r, g, b) = if light {
                (0x26, 0xa6, 0x9a)
            } else {
                (0x24, 0x2a, 0x32)
            };
            rgba.extend_from_slice(&[r, g, b, 0xff]);
        }
    }
    rgba
}

pub struct TrayItem;

impl TrayItem {
    pub const OPEN_SETTINGS: &str = "open-settings";
    pub const RELOAD_SETTINGS: &str = "reload-settings";
    pub const OPEN_LOG: &str = "open-log";
    pub const QUIT: &str = "quit";
}

pub fn init() -> Result<(Tray, impl iced::futures::Stream<Item = TrayEvent>)> {
    let (sink, rx) = trayinit::channel();

    let icon =
        Icon::from_rgba(checker_icon_rgba(32), 32, 32).context("failed to create tray icon")?;
    let state = TrayState::new()
        .with_title("EchoInput")
        .with_icon(icon)
        .with_tooltip("EchoInput")
        .with_menu(Menu::new([
            MenuNode::item(TrayItem::OPEN_SETTINGS, "Open Settings"),
            MenuNode::item(TrayItem::RELOAD_SETTINGS, "Reload Settings"),
            MenuNode::item(TrayItem::OPEN_LOG, "Open Log"),
            MenuNode::separator(),
            MenuNode::item(TrayItem::QUIT, "Quit"),
        ]));
    let tray = Tray::new(state, sink).context("failed to create system tray")?;

    let stream = iced::stream::channel(
        20,
        |mut output: iced::futures::channel::mpsc::Sender<TrayEvent>| async move {
            std::thread::spawn(move || {
                while let Ok(event) = rx.recv() {
                    if let Err(e) = output.try_send(event)
                        && e.is_disconnected()
                    {
                        break;
                    }
                }
            });
        },
    );

    Ok((tray, stream))
}
