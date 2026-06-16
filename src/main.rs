mod fonts;
mod input;
mod keystrokes;
mod settings;
mod ui;

use std::time::{Duration, Instant};

use global_hotkey::hotkey::{CMD_OR_CTRL, Code, HotKey, Modifiers as HotKeyModifiers};
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState};
use iced::theme::Base;
use iced::window::{self, Level, Position};
use iced::{Color, Element, Point, Size, Subscription, Task, Theme};
use rdev::Event;

use crate::fonts::ICON_FONT_DATA;
use crate::input::{InputEvent, InputNormalizer};
use crate::keystrokes::{KeystrokeState, Modifiers};
use crate::settings::{AppSettings, MAX_HISTORY_LIMIT, MIN_HISTORY_LIMIT};
use crate::ui::OverlayLayout;

const TICK_INTERVAL: Duration = Duration::from_millis(100);

const HOTKEY_ID_SETTINGS: u32 = 1;
const HOTKEY_ID_QUIT: u32 = 2;

pub fn main() -> iced::Result {
    iced::daemon(App::boot, App::update, App::view)
        .theme(|_: &App, _window| Theme::Dark)
        .style(|_: &App, theme: &Theme| {
            let mut style = theme.base();
            style.background_color = Color::TRANSPARENT;
            style
        })
        .font(ICON_FONT_DATA)
        .title(App::title)
        .subscription(App::subscription)
        .run()
}

fn overlay_window_settings(layout: OverlayLayout, history_limit: usize) -> window::Settings {
    let window_size = layout.window_size(history_limit);
    let settings = window::Settings {
        size: window_size,
        position: Position::Specific(Point::new(layout.screen_margin, layout.screen_margin)),
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

fn settings_window_settings() -> window::Settings {
    window::Settings {
        size: Size::new(360.0, 150.0),
        min_size: Some(Size::new(320.0, 140.0)),
        position: Position::Centered,
        resizable: true,
        decorations: true,
        transparent: false,
        ..Default::default()
    }
}

struct App {
    overlay_window_id: window::Id,
    settings_window_id: Option<window::Id>,
    hotkey_manager: Option<GlobalHotKeyManager>,
    settings: AppSettings,
    layout: OverlayLayout,
    input: InputNormalizer,
    keystrokes: KeystrokeState,
    held_modifiers: Modifiers,
}

#[derive(Debug, Clone)]
enum Message {
    OverlayOpened(window::Id),
    SettingsOpened(window::Id),
    WindowClosed(window::Id),
    MonitorSize(window::Id, Option<Size>),
    InputHookEvent(Event),
    HotkeyEvent(GlobalHotKeyEvent),
    DecreaseHistoryLimit,
    IncreaseHistoryLimit,
    Tick(Instant),
}

impl App {
    fn boot() -> (Self, Task<Message>) {
        let settings = settings::load();
        let layout = OverlayLayout::default();
        let hotkey_manager = match register_hotkeys() {
            Ok(manager) => Some(manager),
            Err(err) => {
                eprintln!("Failed to register global hotkeys: {err:#}");
                None
            },
        };
        let (overlay_window_id, open_overlay) =
            window::open(overlay_window_settings(layout, settings.history_limit));

        (
            Self {
                overlay_window_id,
                settings_window_id: None,
                hotkey_manager,
                settings,
                layout,
                input: InputNormalizer::default(),
                keystrokes: KeystrokeState::new(settings.history_limit),
                held_modifiers: Modifiers::default(),
            },
            open_overlay.map(Message::OverlayOpened),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::OverlayOpened(id) => Task::batch(vec![
                window::enable_mouse_passthrough(id),
                window::monitor_size(id)
                    .map(move |monitor_size| Message::MonitorSize(id, monitor_size)),
            ]),
            Message::SettingsOpened(id) => window::gain_focus(id),
            Message::WindowClosed(id) => {
                if id == self.overlay_window_id {
                    return iced::exit();
                }

                if self.settings_window_id == Some(id) {
                    self.settings_window_id = None;
                }

                Task::none()
            },
            Message::DecreaseHistoryLimit => {
                self.set_history_limit(self.settings.history_limit.saturating_sub(1))
            },
            Message::IncreaseHistoryLimit => {
                self.set_history_limit(self.settings.history_limit + 1)
            },
            Message::MonitorSize(id, Some(monitor_size)) => {
                let window_size = clamped_window_size(
                    self.layout.window_size(self.settings.history_limit),
                    monitor_size,
                    self.layout.screen_margin,
                );
                let position =
                    bottom_left_position(window_size, monitor_size, self.layout.screen_margin);

                Task::batch(vec![
                    window::resize(id, window_size),
                    window::move_to(id, position),
                ])
            },
            Message::MonitorSize(_, None) => Task::none(),
            Message::InputHookEvent(event) => {
                match self.input.handle_event(event) {
                    Some(InputEvent::Keystroke(keystroke)) => {
                        self.keystrokes.handle_keystroke(keystroke, Instant::now());
                    },
                    Some(InputEvent::ModifiersChanged(modifiers)) => {
                        self.held_modifiers = modifiers;
                    },
                    None => {},
                }

                Task::none()
            },
            Message::HotkeyEvent(event) => {
                if event.state != HotKeyState::Released {
                    return Task::none();
                }

                self.reset_modifiers();
                match event.id {
                    HOTKEY_ID_SETTINGS => self.open_or_focus_settings(),
                    HOTKEY_ID_QUIT => iced::exit(),
                    _ => {
                        eprintln!("Unrecognized hotkey event: id={}", event.id);
                        Task::none()
                    },
                }
            },
            Message::Tick(now) => {
                self.keystrokes.finalize_if_inactive(now);
                self.keystrokes.prune_expired(now);
                Task::none()
            },
        }
    }

    fn view(&self, window: window::Id) -> Element<'_, Message> {
        if window == self.overlay_window_id {
            ui::overlay(&self.keystrokes, self.held_modifiers, self.layout)
        } else if self.settings_window_id == Some(window) {
            ui::settings_window(
                self.settings.history_limit,
                (self.settings.history_limit > MIN_HISTORY_LIMIT)
                    .then_some(Message::DecreaseHistoryLimit),
                (self.settings.history_limit < MAX_HISTORY_LIMIT)
                    .then_some(Message::IncreaseHistoryLimit),
            )
        } else {
            ui::empty_window()
        }
    }

    fn title(&self, window: window::Id) -> String {
        if self.settings_window_id == Some(window) {
            String::from("Settings")
        } else {
            String::from("EchoInput")
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        let mut subscriptions = vec![
            window::close_events().map(Message::WindowClosed),
            Subscription::run(input::global_input_listener).map(Message::InputHookEvent),
            iced::time::every(TICK_INTERVAL).map(Message::Tick),
        ];

        if self.hotkey_manager.is_some() {
            subscriptions
                .push(Subscription::run(global_hotkey_listener).map(Message::HotkeyEvent));
        }

        Subscription::batch(subscriptions)
    }

    fn open_or_focus_settings(&mut self) -> Task<Message> {
        if let Some(id) = self.settings_window_id {
            return window::gain_focus(id);
        }

        let (id, open_settings) = window::open(settings_window_settings());
        self.settings_window_id = Some(id);
        open_settings.map(Message::SettingsOpened)
    }

    fn set_history_limit(&mut self, history_limit: usize) -> Task<Message> {
        let history_limit = history_limit.clamp(MIN_HISTORY_LIMIT, MAX_HISTORY_LIMIT);

        if history_limit == self.settings.history_limit {
            return Task::none();
        }

        self.settings.history_limit = history_limit;
        self.keystrokes.set_history_limit(history_limit);
        settings::save(self.settings);

        let overlay_window_id = self.overlay_window_id;
        window::monitor_size(overlay_window_id)
            .map(move |monitor_size| Message::MonitorSize(overlay_window_id, monitor_size))
    }

    fn reset_modifiers(&mut self) {
        self.input.reset_modifiers();
        self.held_modifiers = Modifiers::default();
    }
}

fn global_hotkey_listener() -> impl iced::futures::Stream<Item = GlobalHotKeyEvent> {
    use iced::futures::SinkExt;

    iced::stream::channel(
        20,
        |mut output: iced::futures::channel::mpsc::Sender<GlobalHotKeyEvent>| async move {
            let receiver = GlobalHotKeyEvent::receiver().clone();
            let (sender, mut events) = tokio::sync::mpsc::unbounded_channel();

            std::thread::spawn(move || {
                while let Ok(event) = receiver.recv() {
                    let _ = sender.send(event);
                }
            });

            while let Some(event) = events.recv().await {
                let _ = output.send(event).await;
            }
        },
    )
}

fn register_hotkeys() -> anyhow::Result<GlobalHotKeyManager> {
    let manager = GlobalHotKeyManager::new()?;

    manager.register(HotKey {
        id: HOTKEY_ID_SETTINGS,
        mods: CMD_OR_CTRL | HotKeyModifiers::SHIFT,
        key: Code::Comma,
    })?;
    manager.register(HotKey {
        id: HOTKEY_ID_QUIT,
        mods: CMD_OR_CTRL | HotKeyModifiers::SHIFT,
        key: Code::KeyQ,
    })?;

    Ok(manager)
}

fn clamped_window_size(desired: Size, monitor_size: Size, screen_margin: f32) -> Size {
    let max_width = (monitor_size.width - screen_margin * 2.0).max(1.0);
    let max_height = (monitor_size.height - screen_margin * 2.0).max(1.0);

    Size::new(desired.width.min(max_width), desired.height.min(max_height))
}

fn bottom_left_position(window_size: Size, monitor_size: Size, screen_margin: f32) -> Point {
    Point::new(
        screen_margin,
        (monitor_size.height - window_size.height - screen_margin).max(screen_margin),
    )
}
