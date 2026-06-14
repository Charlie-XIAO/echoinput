use iced::widget::{Column, Row, container, text};
use iced::{Background, Border, Color, Element, Font, Theme, alignment, border, padding};

use crate::fonts::{ICON_FONT, ICON_KBD_ALT, ICON_KBD_CONTROL, ICON_KBD_META, ICON_KBD_SHIFT};
use crate::keystrokes::{
    BubbleKind, BubblePart, KeyLabelPart, KeystrokeState, Modifiers, key_label_parts,
};

pub fn overlay<'a, Message: 'a>(
    keystrokes: &'a KeystrokeState,
    held_modifiers: Modifiers,
) -> Element<'a, Message> {
    let mut keystroke_list = Column::new()
        .spacing(8)
        .align_x(alignment::Horizontal::Left);

    for bubble in &keystrokes.history {
        let mut row = Row::new().spacing(4).align_y(alignment::Vertical::Center);

        for part in &bubble.parts {
            match part {
                BubblePart::Text(text) => {
                    row = row.push(key_part(KeyLabelPart::Text(text.clone())));
                },
                BubblePart::Key(keystroke) => {
                    for label_part in key_label_parts(keystroke) {
                        row = row.push(key_part(label_part));
                    }
                },
            }
        }

        if bubble.count > 1 {
            row = row.push(
                container(
                    text(format!("×{}", bubble.count))
                        .font(Font::MONOSPACE)
                        .size(18)
                        .color(Color::from_rgba(1.0, 1.0, 1.0, 0.58)),
                )
                .padding(padding::left(4)),
            );
        }

        keystroke_list = keystroke_list.push(event_bubble(row, bubble.kind));
    }

    if !keystrokes.active.is_empty() {
        let row = Row::new()
            .spacing(4)
            .align_y(alignment::Vertical::Center)
            .push(key_part(KeyLabelPart::Text(keystrokes.active.clone())));

        keystroke_list = keystroke_list.push(event_bubble(row, BubbleKind::Typing));
    }

    keystroke_list = keystroke_list.push(modifier_row(held_modifiers));

    container(keystroke_list)
        .width(iced::Length::Fill)
        .height(iced::Length::Fill)
        .padding([80, 40])
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
) -> Element<'a, Message> {
    container(content)
        .padding([4, 10])
        .max_width(520)
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

fn modifier_row<'a, Message: 'a>(modifiers: Modifiers) -> Element<'a, Message> {
    Row::new()
        .spacing(6)
        .padding(padding::top(6))
        .push(modifier_key(ICON_KBD_CONTROL, modifiers.control))
        .push(modifier_key(ICON_KBD_ALT, modifiers.alt))
        .push(modifier_key(ICON_KBD_SHIFT, modifiers.shift))
        .push(modifier_key(ICON_KBD_META, modifiers.meta))
        .into()
}

fn modifier_key<'a, Message: 'a>(label: char, active: bool) -> Element<'a, Message> {
    container(text(label).font(ICON_FONT).size(18))
        .padding([4, 8])
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

fn key_part<'a, Message: 'a>(part: KeyLabelPart) -> Element<'a, Message> {
    match part {
        KeyLabelPart::Text(content) => text(content).font(Font::MONOSPACE).size(24).into(),
        KeyLabelPart::Icon(content) => text(content).font(ICON_FONT).size(24).into(),
    }
}
