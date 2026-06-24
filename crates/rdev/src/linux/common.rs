use crate::rdev::{EventType, Key};

/// Checks if a keyboard LED is active by reading the sysfs brightness files.
///
/// Example targets: "::numlock", "::capslock".
pub fn is_led_on(name: &str) -> bool {
    let Ok(entries) = std::fs::read_dir("/sys/class/leds") else {
        return false;
    };

    for entry in entries {
        let Ok(entry) = entry else {
            continue;
        };

        let entry_name = entry.file_name().into_string().unwrap_or_default();
        if entry_name.ends_with(name) {
            let path = entry.path().join("brightness");
            if let Ok(content) = std::fs::read_to_string(&path)
                && let Ok(brightness) = content.trim().parse::<i32>()
                && brightness > 0
            {
                return true;
            }
        }
    }

    false
}

/// Fallback for numpad keys to standard navigation keys when numlock is off.
pub fn handle_numpad_fallback(event_type: &mut EventType) {
    let (EventType::KeyPress(key) | EventType::KeyRelease(key)) = event_type else {
        return;
    };
    let nav_key = match *key {
        Key::Kp0 => Key::Insert,
        Key::Kp1 => Key::End,
        Key::Kp2 => Key::DownArrow,
        Key::Kp3 => Key::PageDown,
        Key::Kp4 => Key::LeftArrow,
        Key::Kp6 => Key::RightArrow,
        Key::Kp7 => Key::Home,
        Key::Kp8 => Key::UpArrow,
        Key::Kp9 => Key::PageUp,
        Key::KpDelete => Key::Delete,
        k => k,
    };
    *key = nav_key;
}
