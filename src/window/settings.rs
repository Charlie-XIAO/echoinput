use iced::{Size, window};

/// Returns the settings for the settings window.
pub fn settings() -> window::Settings {
    window::Settings {
        size: Size::new(800.0, 600.0),
        min_size: Some(Size::new(400.0, 300.0)),
        ..Default::default()
    }
}
