use std::time::{Duration, Instant};

use iced::futures::StreamExt;
use iced::theme::Base;
use iced::{Color, Element, Subscription, Task, Theme, window};
use trayinit::{Tray, TrayEvent};

use crate::input::{GlobalInputEvent, InputNormalizer};
use crate::keystrokes::KeystrokeState;
use crate::settings::{Settings, SettingsAction, SettingsForm, SettingsMessage};
use crate::tray::TrayItem;
use crate::ui::keystroke::KeystrokeView;
use crate::ui::settings::SettingsView;

const TICK_INTERVAL: Duration = Duration::from_millis(100);

pub fn run() -> iced::Result {
    #[cfg(target_os = "macos")]
    unsafe {
        set_macos_activation_policy();
    }

    iced::daemon(App::boot, App::update, App::view)
        .title("EchoInput")
        .theme(Theme::Dark)
        .style(|_: &App, theme: &Theme| {
            let mut style = theme.base();
            style.background_color = Color::TRANSPARENT;
            style
        })
        .font(include_bytes!("../assets/fonts/echoinput-icons.ttf"))
        .subscription(App::subscription)
        .run()
}

#[cfg(target_os = "macos")]
unsafe fn set_macos_activation_policy() {
    use objc2::MainThreadMarker;
    use objc2_app_kit::{NSApplication, NSApplicationActivationPolicy};

    if let Some(mtm) = MainThreadMarker::new() {
        let app = NSApplication::sharedApplication(mtm);
        let _ = app.setActivationPolicy(NSApplicationActivationPolicy::Accessory);
    }
}

struct App {
    keystroke_window: KeystrokeWindow,
    settings_window: Option<SettingsWindow>,
    settings: Settings,
    input: InputNormalizer,
    _tray: Option<Tray>,
}

#[derive(Debug, Clone)]
enum Message {
    WindowOpened(window::Id),
    WindowClosed(window::Id),
    KeystrokeWindowMonitorSize {
        monitor_size: Option<iced::Size>,
        resize: bool,
    },
    InputEvent(GlobalInputEvent),
    TrayEvent(TrayEvent),
    Tick(Instant),
    Settings(SettingsMessage),
}

struct KeystrokeWindow {
    id: window::Id,
    state: KeystrokeState,
    view: KeystrokeView,
}

struct SettingsWindow {
    id: window::Id,
    form: SettingsForm,
    view: SettingsView,
}

impl App {
    fn boot() -> (Self, Task<Message>) {
        if let Err(e) = crate::logging::init() {
            eprintln!("failed to initialize logging: {e:#}");
        }

        let settings = match crate::settings::load() {
            Ok(settings) => settings,
            Err(e) => {
                log::error!("failed to load settings: {e:#}; using defaults");
                Settings::default()
            },
        };

        let keystroke_view = KeystrokeView::default();
        let keystrokes = KeystrokeState::new(settings.history_limit);

        let mut tasks = Vec::new();

        let tray = match crate::tray::init() {
            Ok((tray, stream)) => {
                tasks.push(Task::stream(stream.map(Message::TrayEvent)));
                Some(tray)
            },
            Err(e) => {
                log::error!("failed to initialize system tray: {e:#}");
                None
            },
        };

        let keystroke_window_settings = crate::window::keystroke::settings(
            keystroke_view.content_size(settings.history_limit),
            &settings.placement,
        );
        let (keystroke_window_id, open_keystroke_window) = window::open(keystroke_window_settings);
        tasks.push(open_keystroke_window.map(Message::WindowOpened));

        (
            Self {
                keystroke_window: KeystrokeWindow {
                    id: keystroke_window_id,
                    state: keystrokes,
                    view: keystroke_view,
                },
                settings_window: None,
                settings,
                input: InputNormalizer::default(),
                _tray: tray,
            },
            Task::batch(tasks),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::WindowOpened(id) => {
                if id == self.keystroke_window.id {
                    Task::batch(vec![
                        window::enable_mouse_passthrough(id),
                        window::monitor_size(id).map(|monitor_size| {
                            Message::KeystrokeWindowMonitorSize {
                                monitor_size,
                                resize: false,
                            }
                        }),
                        #[cfg(target_os = "linux")]
                        crate::window::configure_keystroke_x11_window(id),
                        #[cfg(target_os = "macos")]
                        crate::window::configure_keystroke_macos_window(id),
                    ])
                } else {
                    let settings_window = self
                        .settings_window
                        .as_ref()
                        .expect("unexpected window opened");
                    assert_eq!(id, settings_window.id, "unexpected window id: {id}");
                    window::gain_focus(id)
                }
            },
            Message::WindowClosed(id) => {
                if id == self.keystroke_window.id {
                    iced::exit()
                } else if self
                    .settings_window
                    .as_ref()
                    .is_some_and(|settings_window| id == settings_window.id)
                {
                    self.settings_window = None;
                    Task::none()
                } else {
                    panic!("unexpected window id: {id}");
                }
            },
            Message::KeystrokeWindowMonitorSize {
                monitor_size: Some(monitor_size),
                resize,
            } => {
                let size = self
                    .keystroke_window
                    .view
                    .content_size(self.settings.history_limit);
                let position = crate::window::keystroke::position(
                    size,
                    monitor_size,
                    &self.settings.placement,
                );
                if resize {
                    Task::batch([
                        window::resize(self.keystroke_window.id, size),
                        window::move_to(self.keystroke_window.id, position),
                    ])
                } else {
                    window::move_to(self.keystroke_window.id, position)
                }
            },
            Message::KeystrokeWindowMonitorSize {
                monitor_size: None, ..
            } => {
                log::warn!("failed to get monitor size");
                Task::none()
            },
            Message::InputEvent(event) => {
                let event = match event {
                    GlobalInputEvent::Event(event) => event,
                    GlobalInputEvent::ListenerFailed(e) => {
                        log::error!("failed to start global input listener: {e}");
                        return Task::none();
                    },
                };

                if let Some(event) = self.input.handle_event(event) {
                    self.keystroke_window.state.handle(event, Instant::now());
                }

                Task::none()
            },
            Message::TrayEvent(event) => {
                let TrayEvent::MenuItemActivated { item_id, .. } = event else {
                    return Task::none();
                };
                match item_id.as_str() {
                    TrayItem::SETTINGS => self.open_settings_window(),
                    TrayItem::LOGS => {
                        if let Err(e) = crate::logging::open() {
                            log::warn!("failed to open log file: {e:#}");
                        }
                        Task::none()
                    },
                    TrayItem::QUIT => iced::exit(),
                    _ => {
                        log::warn!("unrecognized tray menu item: {item_id}");
                        Task::none()
                    },
                }
            },
            Message::Tick(now) => {
                self.keystroke_window.state.finalize_if_inactive(now);
                self.keystroke_window.state.prune_expired(now);
                Task::none()
            },
            Message::Settings(message) => self.update_settings(message),
        }
    }

    fn view(&self, window: window::Id) -> Element<'_, Message> {
        if window == self.keystroke_window.id {
            return self
                .keystroke_window
                .view
                .view(&self.keystroke_window.state, &self.settings.placement);
        }

        let settings_window = self
            .settings_window
            .as_ref()
            .expect("unexpected settings window view");
        assert_eq!(window, settings_window.id, "unexpected window id: {window}");
        settings_window
            .view
            .view(&settings_window.form)
            .map(Message::Settings)
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::batch(vec![
            window::close_events().map(Message::WindowClosed),
            Subscription::run(crate::input::listener).map(Message::InputEvent),
            iced::time::every(TICK_INTERVAL).map(Message::Tick),
        ])
    }

    fn apply_settings(&mut self, settings: Settings) -> Task<Message> {
        let old_settings = std::mem::replace(&mut self.settings, settings);
        let history_limit_changed = self.settings.history_limit != old_settings.history_limit;
        let placement_changed = self.settings.placement != old_settings.placement;

        if history_limit_changed {
            self.keystroke_window
                .state
                .set_history_limit(self.settings.history_limit);
        }

        if history_limit_changed || placement_changed {
            return window::monitor_size(self.keystroke_window.id).map(move |monitor_size| {
                Message::KeystrokeWindowMonitorSize {
                    monitor_size,
                    resize: history_limit_changed,
                }
            });
        }

        Task::none()
    }

    fn open_settings_window(&mut self) -> Task<Message> {
        if let Some(settings_window) = &self.settings_window {
            return window::gain_focus(settings_window.id);
        }

        let settings_window_settings = crate::window::settings::settings();
        let (id, open_settings_window) = window::open(settings_window_settings);
        self.settings_window = Some(SettingsWindow {
            id,
            form: SettingsForm::new(&self.settings),
            view: SettingsView::default(),
        });
        open_settings_window.map(Message::WindowOpened)
    }

    fn update_settings(&mut self, message: SettingsMessage) -> Task<Message> {
        let action = self
            .settings_window
            .as_mut()
            .expect("settings message without settings window")
            .form
            .update(message);

        match action {
            Some(SettingsAction::Save(settings)) => {
                if let Err(e) = crate::settings::save(&settings) {
                    log::error!("failed to save settings: {e:#}");
                    return Task::none();
                }

                self.settings_window
                    .as_mut()
                    .expect("settings message without settings window")
                    .form = SettingsForm::new(&settings);

                self.apply_settings(settings)
            },
            Some(SettingsAction::EditJson) => {
                if let Err(e) = crate::settings::open() {
                    log::error!("failed to open settings file: {e:#}");
                }
                Task::none()
            },
            None => Task::none(),
        }
    }
}
