use std::borrow::Cow;

use iced::widget::{Column, Row, container, text};
use iced::{Background, Color, Element, Font, Size, Theme, alignment, padding};

use crate::icons::Icon;
use crate::keystrokes::{
    BubbleKind, BubblePart, KeyLabel, KeystrokeState, MAX_ACTIVE_TEXT_LEN, Modifiers,
};
use crate::ui::style::{self, DesignTokens, ICON_FONT};

const MONOSPACE_CHAR_WIDTH_RATIO: f32 = 0.62;
const REPEAT_COUNT_CHAR_BUDGET: f32 = 3.0;
const MODIFIER_ROW_MAX_ITEMS: f32 = 5.0;

#[derive(Debug)]
pub struct KeystrokeLayout {
    tokens: DesignTokens,
    event_font_size: f32,
    repeat_count_font_size: f32,
    modifier_font_size: f32,
    row_spacing: f32,
    column_spacing: f32,
    event_padding_vertical: f32,
    event_padding_horizontal: f32,
    modifier_padding_vertical: f32,
    modifier_padding_horizontal: f32,
    modifier_top_padding: f32,
    width_slack: f32,
}

impl Default for KeystrokeLayout {
    fn default() -> Self {
        Self {
            tokens: DesignTokens::dark(),
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
        }
    }
}

impl KeystrokeLayout {
    pub fn content_size(&self, history_limit: usize) -> Size {
        Size::new(self.stack_width(), self.stack_height(history_limit))
    }

    fn stack_width(&self) -> f32 {
        self.event_max_width().max(self.modifier_row_max_width())
    }

    fn event_max_width(&self) -> f32 {
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

        let spacing = self.column_spacing * (history_limit as f32);

        (event_row_height * (history_limit as f32) + modifier_row_height + spacing).ceil()
    }

    fn row_height(&self, font_size: f32, vertical_padding: f32) -> f32 {
        font_size * self.tokens.typography.line_height + 2.0 * vertical_padding
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
                            .line_height(self.tokens.typography.line_height)
                            .color(style::with_alpha(self.tokens.colors.muted_fg, 0.8)),
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
                let (bg, fg, border) = self.bubble_colors(kind);

                container::Style {
                    text_color: Some(fg),
                    background: Some(Background::Color(bg)),
                    border: style::border(border, self.tokens.border_width, self.tokens.radius),
                    ..Default::default()
                }
            })
            .into()
    }

    /// Returns the background, foreground, and border colors of a bubble.
    fn bubble_colors(&self, kind: BubbleKind) -> (Color, Color, Color) {
        let colors = self.tokens.colors;
        match kind {
            BubbleKind::Typing => (
                style::with_alpha(colors.card, 0.88),
                colors.card_fg,
                style::with_alpha(colors.border, 0.72),
            ),
            BubbleKind::Shortcut => (
                style::with_alpha(colors.primary_container, 0.88),
                colors.primary_container_fg,
                style::with_alpha(colors.primary, 0.95),
            ),
            BubbleKind::Special => (
                style::with_alpha(colors.accent_container, 0.88),
                colors.accent_container_fg,
                style::with_alpha(colors.accent, 0.95),
            ),
        }
    }

    fn modifier_row<'a, Message: 'a>(&'a self, modifiers: &'a Modifiers) -> Element<'a, Message> {
        Row::new()
            .spacing(self.row_spacing + 2.0)
            .padding(padding::top(self.modifier_top_padding))
            .push(self.modifier_key(Icon::ChevronUp, modifiers.control))
            .push(self.modifier_key(Icon::Option, modifiers.alt))
            .push(self.modifier_key(Icon::ArrowBigUp, modifiers.shift))
            .push(self.modifier_key(Icon::Command, modifiers.meta))
            .into()
    }

    fn modifier_key<'a, Message: 'a>(&'a self, icon: Icon, active: bool) -> Element<'a, Message> {
        container(
            text(char::from(icon))
                .font(ICON_FONT)
                .size(self.modifier_font_size)
                .line_height(self.tokens.typography.line_height),
        )
        .padding([
            self.modifier_padding_vertical,
            self.modifier_padding_horizontal,
        ])
        .style(move |_: &Theme| {
            let (bg, fg, border) = self.modifier_colors(active);

            container::Style {
                text_color: Some(fg),
                background: Some(Background::Color(bg)),
                border: style::border(border, self.tokens.border_width, self.tokens.radius),
                ..Default::default()
            }
        })
        .into()
    }

    /// Returns the background, foreground, and border colors of a modifier.
    fn modifier_colors(&self, active: bool) -> (Color, Color, Color) {
        let colors = self.tokens.colors;
        if active {
            (
                style::with_alpha(colors.primary_container, 0.88),
                colors.primary_container_fg,
                style::with_alpha(colors.primary, 0.95),
            )
        } else {
            (
                style::with_alpha(colors.muted, 0.52),
                style::with_alpha(colors.muted_fg, 0.82),
                style::with_alpha(colors.border, 0.72),
            )
        }
    }

    fn key_label<'a, Message: 'a>(&'a self, label: KeyLabel<'a>) -> Element<'a, Message> {
        let text = match label {
            KeyLabel::Text(content) => text(content).font(Font::MONOSPACE),
            KeyLabel::Char(ch) => text(ch).font(Font::MONOSPACE),
            KeyLabel::Icon(icon) => text(char::from(icon)).font(ICON_FONT),
        };
        text.size(self.event_font_size)
            .line_height(self.tokens.typography.line_height)
            .into()
    }
}
