use iced::widget::{button, pick_list as pick_list_widget, text_input as text_input_widget};
use iced::{Background, Border, Color, Font, Shadow, border};

pub const ICON_FONT: Font = Font::with_name("echoinput-icons");

#[derive(Debug, Clone, Copy)]
pub struct DesignTokens {
    pub colors: Colors,
    pub typography: Typography,
    pub radius: f32,
    pub border_width: f32,
}

impl DesignTokens {
    pub const fn dark() -> Self {
        Self {
            colors: Colors {
                background: Color::from_rgb8(0x09, 0x09, 0x0b), // shadcn background; zinc-950
                background_fg: Color::from_rgb8(0xfa, 0xfa, 0xfa), // shadcn foreground; zinc-50
                card: Color::from_rgb8(0x18, 0x18, 0x1b),       // zinc-900
                card_fg: Color::from_rgb8(0xfa, 0xfa, 0xfa),    // zinc-50
                primary: Color::from_rgb8(0x60, 0xa5, 0xfa),    // blue-400
                primary_container: Color::from_rgb8(0x17, 0x25, 0x54), // blue-950
                primary_container_fg: Color::from_rgb8(0xef, 0xf6, 0xff), // blue-50
                accent: Color::from_rgb8(0x22, 0xd3, 0xee),     // cyan-400
                accent_container: Color::from_rgb8(0x08, 0x33, 0x44), // cyan-950
                accent_container_fg: Color::from_rgb8(0xec, 0xfe, 0xff), // cyan-50
                muted: Color::from_rgb8(0x27, 0x27, 0x2a),      // zinc-800
                muted_fg: Color::from_rgb8(0xa1, 0xa1, 0xaa),   // zinc-400
                border: Color::from_rgb8(0x71, 0x71, 0x7a),     // zinc-500
                input: Color::from_rgb8(0x3f, 0x3f, 0x46),      // shadcn input; zinc-700
                destructive: Color::from_rgb8(0xef, 0x44, 0x44), // shadcn destructive; red-500
            },
            typography: Typography { line_height: 1.3 },
            radius: 8.0,
            border_width: 1.0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Colors {
    pub background: Color,
    pub background_fg: Color,
    pub card: Color,
    pub card_fg: Color,
    pub primary: Color,
    pub primary_container: Color,
    pub primary_container_fg: Color,
    pub accent: Color,
    pub accent_container: Color,
    pub accent_container_fg: Color,
    pub muted: Color,
    pub muted_fg: Color,
    pub border: Color,
    pub input: Color,
    pub destructive: Color,
}

#[derive(Debug, Clone, Copy)]
pub struct Typography {
    pub line_height: f32,
}

pub const fn with_alpha(mut color: Color, alpha: f32) -> Color {
    color.a = alpha;
    color
}

pub fn border(color: Color, width: f32, radius: f32) -> Border {
    Border {
        color,
        width,
        radius: border::Radius::from(radius),
    }
}

pub fn text_input(
    tokens: DesignTokens,
    status: text_input_widget::Status,
    invalid: bool,
) -> text_input_widget::Style {
    let colors = tokens.colors;
    let border_color = if invalid {
        colors.destructive
    } else if matches!(status, text_input_widget::Status::Focused { .. }) {
        colors.primary
    } else {
        colors.input
    };

    text_input_widget::Style {
        background: Background::Color(colors.card),
        border: border(border_color, tokens.border_width, tokens.radius),
        icon: colors.muted_fg,
        placeholder: colors.muted_fg,
        value: colors.card_fg,
        selection: with_alpha(colors.primary, 0.35),
    }
}

pub fn pick_list(
    tokens: DesignTokens,
    status: pick_list_widget::Status,
) -> pick_list_widget::Style {
    let colors = tokens.colors;
    let border_color = if matches!(status, pick_list_widget::Status::Opened { .. }) {
        colors.primary
    } else {
        colors.input
    };

    pick_list_widget::Style {
        text_color: colors.card_fg,
        placeholder_color: colors.muted_fg,
        handle_color: colors.muted_fg,
        background: Background::Color(colors.card),
        border: border(border_color, tokens.border_width, tokens.radius),
    }
}

pub fn primary_button(tokens: DesignTokens, status: button::Status) -> button::Style {
    let colors = tokens.colors;
    let background = match status {
        button::Status::Disabled => with_alpha(colors.muted, 0.8),
        button::Status::Hovered => with_alpha(colors.primary, 0.85),
        button::Status::Pressed => with_alpha(colors.primary, 0.7),
        button::Status::Active => colors.primary,
    };

    button::Style {
        background: Some(Background::Color(background)),
        text_color: colors.primary_container,
        border: border(background, tokens.border_width, tokens.radius),
        shadow: Shadow::default(),
        snap: true,
    }
}

pub fn secondary_button(tokens: DesignTokens, status: button::Status) -> button::Style {
    let colors = tokens.colors;
    let background = match status {
        button::Status::Disabled => Color::TRANSPARENT,
        button::Status::Hovered => colors.muted,
        button::Status::Pressed => with_alpha(colors.muted, 0.7),
        button::Status::Active => Color::TRANSPARENT,
    };

    button::Style {
        background: Some(Background::Color(background)),
        text_color: colors.card_fg,
        border: border(colors.input, tokens.border_width, tokens.radius),
        shadow: Shadow::default(),
        snap: true,
    }
}
