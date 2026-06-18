use std::borrow::Cow;

use iced::widget::{Column, Row, container, text};
use iced::{Background, Border, Color, Element, Font, Size, Theme, alignment, border, padding};

use crate::icons::{ICON_FONT, Icon};
use crate::keystrokes::{
    BubbleKind, BubblePart, KeyLabel, KeystrokeState, MAX_ACTIVE_TEXT_LEN, Modifiers,
};

const MONOSPACE_CHAR_WIDTH_RATIO: f32 = 0.62;
const REPEAT_COUNT_CHAR_BUDGET: f32 = 3.0;
const MODIFIER_ROW_MAX_ITEMS: f32 = 5.0;

#[derive(Debug)]
pub struct Layout {
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

impl Default for Layout {
    fn default() -> Self {
        Self {
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

impl Layout {
    pub fn content_size(&self, history_limit: usize) -> Size {
        Size::new(self.stack_width(), self.stack_height(history_limit))
    }

    fn stack_width(&self) -> f32 {
        self.event_max_width().max(self.modifier_row_max_width())
    }

    pub fn event_max_width(&self) -> f32 {
        let text_width =
            MAX_ACTIVE_TEXT_LEN as f32 * self.event_font_size * MONOSPACE_CHAR_WIDTH_RATIO;
        let repeat_count_width = self.repeat_count_font_size * REPEAT_COUNT_CHAR_BUDGET;
        let padding = 2.0 * self.event_padding_horizontal;

        (text_width + repeat_count_width + padding + self.width_slack).ceil()
    }

    fn modifier_row_max_width(&self) -> f32 {
        let item_width = self.modifier_font_size + 2.0 * self.modifier_padding_horizontal;
        let spacing = (self.row_spacing + 2.0) * (MODIFIER_ROW_MAX_ITEMS - 1.0);

        (item_width * MODIFIER_ROW_MAX_ITEMS + spacing).ceil()
    }

    fn stack_height(&self, history_limit: usize) -> f32 {
        let event_row_height = self.row_height(self.event_font_size, self.event_padding_vertical);
        let modifier_row_height = self
            .row_height(self.modifier_font_size, self.modifier_padding_vertical)
            + self.modifier_top_padding;

        let row_count = history_limit as f32 + 2.0;
        let spacing = self.column_spacing * (row_count - 1.0);

        (event_row_height * (history_limit as f32 + 1.0) + modifier_row_height + spacing).ceil()
    }

    fn row_height(&self, font_size: f32, vertical_padding: f32) -> f32 {
        font_size * self.text_line_height + 2.0 * vertical_padding
    }

    pub fn view<'a, Message: 'a>(
        &'a self,
        keystrokes: &'a KeystrokeState,
        held_modifiers: &'a Modifiers,
    ) -> Element<'a, Message> {
        let mut keystroke_list = Column::new()
            .spacing(self.column_spacing)
            .align_x(alignment::Horizontal::Left);

        for bubble in &keystrokes.history {
            let mut row = Row::new()
                .spacing(self.row_spacing)
                .align_y(alignment::Vertical::Center);

            for part in &bubble.parts {
                match part {
                    BubblePart::Text(text) => {
                        row = row.push(self.key_label(KeyLabel::Text(Cow::Borrowed(text))));
                    },
                    BubblePart::Key(keystroke) => {
                        for label in keystroke.labels() {
                            row = row.push(self.key_label(label));
                        }
                    },
                }
            }

            if bubble.count > 1 {
                row = row.push(
                    container(
                        text(format!("×{}", bubble.count))
                            .font(Font::MONOSPACE)
                            .size(self.repeat_count_font_size)
                            .line_height(self.text_line_height)
                            .color(Color::from_rgba(1.0, 1.0, 1.0, 0.58)),
                    )
                    .padding(padding::left(4)),
                );
            }

            keystroke_list = keystroke_list.push(self.event_bubble(row, bubble.kind));
        }

        if !keystrokes.active.is_empty() {
            let row = Row::new()
                .spacing(self.row_spacing)
                .align_y(alignment::Vertical::Center)
                .push(self.key_label(KeyLabel::Text(Cow::Borrowed(&keystrokes.active))));

            keystroke_list = keystroke_list.push(self.event_bubble(row, BubbleKind::Typing));
        }

        keystroke_list = keystroke_list.push(self.modifier_row(held_modifiers));

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
        &'a self,
        content: Row<'a, Message>,
        kind: BubbleKind,
    ) -> Element<'a, Message> {
        container(content)
            .padding([self.event_padding_vertical, self.event_padding_horizontal])
            .max_width(self.event_max_width())
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

    fn modifier_row<'a, Message: 'a>(&'a self, modifiers: &'a Modifiers) -> Element<'a, Message> {
        Row::new()
            .spacing(self.row_spacing + 2.0)
            .padding(padding::top(self.modifier_top_padding))
            .push(self.modifier_key(Icon::KbdControl, modifiers.control))
            .push(self.modifier_key(Icon::KbdAlt, modifiers.alt))
            .push(self.modifier_key(Icon::KbdShift, modifiers.shift))
            .push(self.modifier_key(Icon::KbdMeta, modifiers.meta))
            .into()
    }

    fn modifier_key<'a, Message: 'a>(&'a self, icon: Icon, active: bool) -> Element<'a, Message> {
        container(
            text(icon.codepoint())
                .font(ICON_FONT)
                .size(self.modifier_font_size)
                .line_height(self.text_line_height),
        )
        .padding([
            self.modifier_padding_vertical,
            self.modifier_padding_horizontal,
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

    fn key_label<'a, Message: 'a>(&'a self, label: KeyLabel<'a>) -> Element<'a, Message> {
        let text = match label {
            KeyLabel::Text(content) => text(content).font(Font::MONOSPACE),
            KeyLabel::Char(ch) => text(ch).font(Font::MONOSPACE),
            KeyLabel::Icon(icon) => text(icon.codepoint()).font(ICON_FONT),
        };
        text.size(self.event_font_size)
            .line_height(self.text_line_height)
            .into()
    }
}
