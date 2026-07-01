use anyhow::{Context, Result};
use global_hotkey::hotkey::{CMD_OR_CTRL, Code, HotKey, Modifiers as HotKeyModifiers};
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager};

pub struct HotKeyId;

impl HotKeyId {
    pub const OPEN_SETTINGS: u32 = 0;
    pub const RELOAD_SETTINGS: u32 = 1;
    pub const OPEN_LOG: u32 = 2;
    pub const QUIT: u32 = 3;
}

/// Initialize global hotkeys.
pub fn init() -> Result<GlobalHotKeyManager> {
    let manager = GlobalHotKeyManager::new().context("failed to create global hotkey manager")?;

    manager
        .register(HotKey {
            id: HotKeyId::OPEN_SETTINGS,
            mods: CMD_OR_CTRL | HotKeyModifiers::SHIFT,
            key: Code::Comma,
        })
        .context("failed to register open-settings hotkey")?;

    manager
        .register(HotKey {
            id: HotKeyId::RELOAD_SETTINGS,
            mods: CMD_OR_CTRL | HotKeyModifiers::SHIFT,
            key: Code::KeyR,
        })
        .context("failed to register reload-setting hotkey")?;

    manager
        .register(HotKey {
            id: HotKeyId::OPEN_LOG,
            mods: CMD_OR_CTRL | HotKeyModifiers::SHIFT,
            key: Code::KeyL,
        })
        .context("failed to register open-log hotkey")?;

    manager
        .register(HotKey {
            id: HotKeyId::QUIT,
            mods: CMD_OR_CTRL | HotKeyModifiers::SHIFT,
            key: Code::KeyQ,
        })
        .context("failed to register quit hotkey")?;

    Ok(manager)
}

/// Listener for global hotkey events.
pub fn listener() -> impl iced::futures::Stream<Item = GlobalHotKeyEvent> {
    use iced::futures::SinkExt;

    iced::stream::channel(
        20,
        |mut output: iced::futures::channel::mpsc::Sender<GlobalHotKeyEvent>| async move {
            let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();

            GlobalHotKeyEvent::set_event_handler(Some(move |event| {
                let _ = tx.send(event);
            }));

            while let Some(event) = rx.recv().await {
                if let Err(e) = output.send(event).await
                    && e.is_disconnected()
                {
                    break;
                }
            }
        },
    )
}
