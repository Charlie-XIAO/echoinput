use iced::{Border, Color, Font, border};

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
                card: Color::from_rgb8(0x18, 0x18, 0x1b),    // zinc-900
                card_fg: Color::from_rgb8(0xfa, 0xfa, 0xfa), // zinc-50
                primary: Color::from_rgb8(0x60, 0xa5, 0xfa), // blue-400
                primary_container: Color::from_rgb8(0x17, 0x25, 0x54), // blue-950
                primary_container_fg: Color::from_rgb8(0xef, 0xf6, 0xff), // blue-50
                accent: Color::from_rgb8(0x22, 0xd3, 0xee),  // cyan-400
                accent_container: Color::from_rgb8(0x08, 0x33, 0x44), // cyan-950
                accent_container_fg: Color::from_rgb8(0xec, 0xfe, 0xff), // cyan-50
                muted: Color::from_rgb8(0x27, 0x27, 0x2a),   // zinc-800
                muted_fg: Color::from_rgb8(0xa1, 0xa1, 0xaa), // zinc-400
                border: Color::from_rgb8(0x71, 0x71, 0x7a),  // zinc-500
            },
            typography: Typography { line_height: 1.3 },
            radius: 8.0,
            border_width: 1.0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Colors {
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
