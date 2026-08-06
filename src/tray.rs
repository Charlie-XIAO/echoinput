use anyhow::{Context, Result};
use image::ImageFormat;
use trayinit::{Icon, Menu, MenuNode, Tray, TrayEvent, TrayState};

pub struct TrayItem;

impl TrayItem {
    pub const SETTINGS: &str = "settings";
    pub const LOGS: &str = "logs";
    pub const QUIT: &str = "quit";
}

pub fn init() -> Result<(Tray, impl iced::futures::Stream<Item = TrayEvent>)> {
    let (sink, rx) = trayinit::channel();

    let icon = {
        let bytes = include_bytes!("../assets/logo.png");
        let img = image::load_from_memory_with_format(bytes, ImageFormat::Png)
            .context("failed to load tray icon")?
            .to_rgba8();
        let (width, height) = img.dimensions();
        Icon::from_rgba(img.into_raw(), width, height).context("failed to create tray icon")?
    };

    let state = TrayState::new()
        .with_icon(icon)
        .with_tooltip("EchoInput")
        .with_menu(Menu::new([
            MenuNode::item(TrayItem::SETTINGS, "Settings"),
            MenuNode::item(TrayItem::LOGS, "Logs"),
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
