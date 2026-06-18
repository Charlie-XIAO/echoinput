mod app;
mod hotkey;
mod icons;
mod input;
mod keystrokes;
mod logging;
mod settings;
mod ui;
mod window;

pub fn main() -> iced::Result {
    app::run()
}
