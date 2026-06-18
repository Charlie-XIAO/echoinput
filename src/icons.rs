use iced::Font;

pub const ICON_FONT_DATA: &[u8] = include_bytes!("../assets/fonts/echoinput-icons.ttf");
pub const ICON_FONT: Font = Font::with_name("echoinput-icons");

#[allow(clippy::enum_variant_names)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Icon {
    KbdAlt,
    KbdBackspace,
    KbdControl,
    KbdDelete,
    KbdArrowDown,
    KbdEnd,
    KbdEscape,
    KbdHome,
    KbdArrowLeft,
    KbdMeta,
    KbdPageDown,
    KbdPageUp,
    KbdEnter,
    KbdArrowRight,
    KbdShift,
    KbdSpace,
    KbdTab,
    KbdArrowUp,
}

impl Icon {
    pub fn codepoint(&self) -> char {
        match self {
            Self::KbdAlt => '\u{ea01}',
            Self::KbdBackspace => '\u{ea02}',
            Self::KbdControl => '\u{ea03}',
            Self::KbdDelete => '\u{ea04}',
            Self::KbdArrowDown => '\u{ea05}',
            Self::KbdEnd => '\u{ea06}',
            Self::KbdEscape => '\u{ea07}',
            Self::KbdHome => '\u{ea08}',
            Self::KbdArrowLeft => '\u{ea09}',
            Self::KbdMeta => '\u{ea0a}',
            Self::KbdPageDown => '\u{ea0b}',
            Self::KbdPageUp => '\u{ea0c}',
            Self::KbdEnter => '\u{ea0d}',
            Self::KbdArrowRight => '\u{ea0e}',
            Self::KbdShift => '\u{ea0f}',
            Self::KbdSpace => '\u{ea10}',
            Self::KbdTab => '\u{ea11}',
            Self::KbdArrowUp => '\u{ea12}',
        }
    }
}
