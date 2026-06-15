use iced::widget::{Column, Row, container, text};
use iced::{Background, Border, Color, Element, Font, Size, Theme, alignment, border, padding};

use crate::fonts::{ICON_FONT, ICON_KBD_ALT, ICON_KBD_CONTROL, ICON_KBD_META, ICON_KBD_SHIFT};
use crate::keystrokes::{
    BubbleKind, BubblePart, KeyLabelPart, KeystrokeState, MAX_ACTIVE_TEXT_LEN, MAX_HISTORY,
    Modifiers, key_label_parts,
};

const MONOSPACE_CHAR_WIDTH_RATIO: f32 = 0.62;
const REPEAT_COUNT_CHAR_BUDGET: f32 = 3.0;

#[derive(Debug, Clone, Copy)]
pub struct OverlayLayout {
    pub screen_margin: f32,
    pub event_font_size: f32,
    pub repeat_count_font_size: f32,
    pub modifier_font_size: f32,
    pub row_spacing: f32,
    pub column_spacing: f32,
    pub event_padding_vertical: f32,
    pub event_padding_horizontal: f32,
    pub modifier_padding_vertical: f32,
    pub modifier_padding_horizontal: f32,
    pub modifier_top_padding: f32,
    pub width_slack: f32,
    pub text_line_height: f32,
}

impl Default for OverlayLayout {
    fn default() -> Self {
        Self {
            screen_margin: 40.0,
            event_font_size: 24.0,
            repeat_count_font_size: 18.0,
            modifier_font_size: 18.0,
            row_spacing: 4.0,
            column_spacing: 8.0,
            event_padding_vertical: 4.0,
            event_padding_horizontal: 10.0,
            modifier_padding_vertical: 4.0,
            modifier_padding_horizontal: 8.0,
            modifier_top_padding: 6.0,
            width_slack: 24.0,
            text_line_height: 1.3,
        }
    }
}

impl OverlayLayout {
    pub fn window_size(self) -> Size {
        Size::new(self.event_max_width(), self.stack_height())
    }

    pub fn event_max_width(self) -> f32 {
        let text_width =
            MAX_ACTIVE_TEXT_LEN as f32 * self.event_font_size * MONOSPACE_CHAR_WIDTH_RATIO;
        let repeat_count_width = self.repeat_count_font_size * REPEAT_COUNT_CHAR_BUDGET;
        let padding = 2.0 * self.event_padding_horizontal;

        (text_width + repeat_count_width + padding + self.width_slack).ceil()
    }

    fn stack_height(self) -> f32 {
        let event_row_height = self.row_height(self.event_font_size, self.event_padding_vertical);
        let modifier_row_height = self
            .row_height(self.modifier_font_size, self.modifier_padding_vertical)
            + self.modifier_top_padding;

        let row_count = MAX_HISTORY as f32 + 2.0;
        let spacing = self.column_spacing * (row_count - 1.0);

        (event_row_height * (MAX_HISTORY as f32 + 1.0) + modifier_row_height + spacing).ceil()
    }

    fn row_height(self, font_size: f32, vertical_padding: f32) -> f32 {
        font_size * self.text_line_height + 2.0 * vertical_padding
    }
}

pub fn overlay<'a, Message: 'a>(
    keystrokes: &'a KeystrokeState,
    held_modifiers: Modifiers,
    layout: OverlayLayout,
) -> Element<'a, Message> {
    let mut keystroke_list = Column::new()
        .spacing(layout.column_spacing)
        .align_x(alignment::Horizontal::Left);

    for bubble in &keystrokes.history {
        let mut row = Row::new()
            .spacing(layout.row_spacing)
            .align_y(alignment::Vertical::Center);

        for part in &bubble.parts {
            match part {
                BubblePart::Text(text) => {
                    row = row.push(key_part(KeyLabelPart::Text(text.clone()), layout));
                },
                BubblePart::Key(keystroke) => {
                    for label_part in key_label_parts(keystroke) {
                        row = row.push(key_part(label_part, layout));
                    }
                },
            }
        }

        if bubble.count > 1 {
            row = row.push(
                container(
                    text(format!("×{}", bubble.count))
                        .font(Font::MONOSPACE)
                        .size(layout.repeat_count_font_size)
                        .line_height(layout.text_line_height)
                        .color(Color::from_rgba(1.0, 1.0, 1.0, 0.58)),
                )
                .padding(padding::left(4)),
            );
        }

        keystroke_list = keystroke_list.push(event_bubble(row, bubble.kind, layout));
    }

    if !keystrokes.active.is_empty() {
        let row = Row::new()
            .spacing(layout.row_spacing)
            .align_y(alignment::Vertical::Center)
            .push(key_part(
                KeyLabelPart::Text(keystrokes.active.clone()),
                layout,
            ));

        keystroke_list = keystroke_list.push(event_bubble(row, BubbleKind::Typing, layout));
    }

    keystroke_list = keystroke_list.push(modifier_row(held_modifiers, layout));

    container(keystroke_list)
        .width(iced::Length::Fill)
        .height(iced::Length::Fill)
        .align_x(alignment::Horizontal::Left)
        .align_y(alignment::Vertical::Bottom)
        .style(|_: &Theme| container::Style {
            background: Some(Background::Color(Color::TRANSPARENT)),
            ..Default::default()
        })
        .into()
}

fn event_bubble<'a, Message: 'a>(
    content: Row<'a, Message>,
    kind: BubbleKind,
    layout: OverlayLayout,
) -> Element<'a, Message> {
    container(content)
        .padding([
            layout.event_padding_vertical,
            layout.event_padding_horizontal,
        ])
        .max_width(layout.event_max_width())
        .style(move |_: &Theme| {
            let (bg_color, border_color) = match kind {
                BubbleKind::Typing => (
                    Color::from_rgba(0.04, 0.07, 0.09, 0.76),
                    Color::from_rgba(0.55, 0.85, 1.0, 0.58),
                ),
                BubbleKind::Shortcut => (
                    Color::from_rgba(0.0, 0.38, 0.55, 0.86),
                    Color::from_rgba(0.0, 0.82, 1.0, 0.72),
                ),
                BubbleKind::Special => (
                    Color::from_rgba(0.26, 0.11, 0.24, 0.84),
                    Color::from_rgba(1.0, 0.26, 0.62, 0.68),
                ),
            };

            container::Style {
                text_color: Some(Color::WHITE),
                background: Some(Background::Color(bg_color)),
                border: Border {
                    radius: border::Radius::from(8),
                    width: 1.0,
                    color: border_color,
                },
                ..Default::default()
            }
        })
        .into()
}

fn modifier_row<'a, Message: 'a>(
    modifiers: Modifiers,
    layout: OverlayLayout,
) -> Element<'a, Message> {
    Row::new()
        .spacing(layout.row_spacing + 2.0)
        .padding(padding::top(layout.modifier_top_padding))
        .push(modifier_key(ICON_KBD_CONTROL, modifiers.control, layout))
        .push(modifier_key(ICON_KBD_ALT, modifiers.alt, layout))
        .push(modifier_key(ICON_KBD_SHIFT, modifiers.shift, layout))
        .push(modifier_key(ICON_KBD_META, modifiers.meta, layout))
        .into()
}

fn modifier_key<'a, Message: 'a>(
    label: char,
    active: bool,
    layout: OverlayLayout,
) -> Element<'a, Message> {
    container(
        text(label)
            .font(ICON_FONT)
            .size(layout.modifier_font_size)
            .line_height(layout.text_line_height),
    )
    .padding([
        layout.modifier_padding_vertical,
        layout.modifier_padding_horizontal,
    ])
    .style(move |_: &Theme| {
        let (text_color, bg_color, border_color) = if active {
            (
                Color::from_rgba(1.0, 1.0, 1.0, 0.95),
                Color::from_rgba(0.0, 0.38, 0.55, 0.72),
                Color::from_rgba(0.0, 0.82, 1.0, 0.68),
            )
        } else {
            (
                Color::from_rgba(1.0, 1.0, 1.0, 0.28),
                Color::from_rgba(0.04, 0.07, 0.09, 0.28),
                Color::from_rgba(1.0, 1.0, 1.0, 0.12),
            )
        };

        container::Style {
            text_color: Some(text_color),
            background: Some(Background::Color(bg_color)),
            border: Border {
                radius: border::Radius::from(8),
                width: 1.0,
                color: border_color,
            },
            ..Default::default()
        }
    })
    .into()
}

fn key_part<'a, Message: 'a>(part: KeyLabelPart, layout: OverlayLayout) -> Element<'a, Message> {
    match part {
        KeyLabelPart::Text(content) => text(content)
            .font(Font::MONOSPACE)
            .size(layout.event_font_size)
            .line_height(layout.text_line_height)
            .into(),
        KeyLabelPart::Icon(content) => text(content)
            .font(ICON_FONT)
            .size(layout.event_font_size)
            .line_height(layout.text_line_height)
            .into(),
    }
}
