use iced::widget::{Column, Row, button, container, pick_list, rule, scrollable, text, text_input};
use iced::{Background, Element, Length, Theme, alignment};

use crate::settings::{PlacementAnchor, SettingsEditor, SettingsMessage};
use crate::ui::style::{self, DesignTokens};

const CONTROL_WIDTH: f32 = 140.0;
const CONTROL_PADDING: f32 = 7.0;
const CONTENT_PADDING: f32 = 24.0;

const ANCHORS: [PlacementAnchor; 4] = [
    PlacementAnchor::BottomLeft,
    PlacementAnchor::BottomRight,
    PlacementAnchor::TopLeft,
    PlacementAnchor::TopRight,
];

#[derive(Debug)]
pub struct SettingsView {
    tokens: DesignTokens,
}

impl Default for SettingsView {
    fn default() -> Self {
        Self {
            tokens: DesignTokens::dark(),
        }
    }
}

impl SettingsView {
    pub fn view<'a>(&'a self, editor: &'a SettingsEditor) -> Element<'a, SettingsMessage> {
        let draft = editor.draft();
        let history_limit = self.number_input(
            &draft.history_limit,
            draft.history_limit_is_valid(),
            SettingsMessage::HistoryLimitChanged,
        );
        let anchor = pick_list(ANCHORS, Some(draft.anchor), SettingsMessage::AnchorChanged)
            .width(CONTROL_WIDTH)
            .padding([CONTROL_PADDING, 10.0])
            .text_size(14)
            .style(move |_, status| style::pick_list(self.tokens, status));
        let margin_x = self.number_input(
            &draft.margin_x,
            draft.margin_x_is_valid(),
            SettingsMessage::MarginXChanged,
        );
        let margin_y = self.number_input(
            &draft.margin_y,
            draft.margin_y_is_valid(),
            SettingsMessage::MarginYChanged,
        );
        let valid = draft.settings().is_some();

        let content = Column::new()
            .spacing(16)
            .push(
                text("Settings")
                    .size(20)
                    .color(self.tokens.colors.background_fg),
            )
            .push(
                Column::new()
                    .spacing(8)
                    .push(self.section_title("Basics"))
                    .push(self.setting_row(
                        "History limit",
                        "Maximum visible keystroke rows, including active typing.",
                        history_limit,
                    )),
            )
            .push(
                Column::new()
                    .spacing(8)
                    .push(self.section_title("Placement"))
                    .push(self.setting_row(
                        "Anchor",
                        "Screen corner that keeps the newest keystroke nearest the edge.",
                        anchor.into(),
                    ))
                    .push(rule::horizontal(1))
                    .push(self.setting_row(
                        "Horizontal margin",
                        "Distance from the anchored horizontal edge in logical pixels.",
                        margin_x,
                    ))
                    .push(rule::horizontal(1))
                    .push(self.setting_row(
                        "Vertical margin",
                        "Distance from the anchored vertical edge in logical pixels.",
                        margin_y,
                    )),
            );

        let edit_json = button(text("Edit in JSON").size(14))
            .padding([CONTROL_PADDING, 12.0])
            .style(move |_, status| style::secondary_button(self.tokens, status))
            .on_press(SettingsMessage::EditJson);
        let mut save = button(text("Save").size(14))
            .padding([CONTROL_PADDING, 14.0])
            .style(move |_, status| style::primary_button(self.tokens, status));
        if valid {
            save = save.on_press(SettingsMessage::Save);
        }

        container(
            Column::new()
                .push(
                    scrollable(container(content).padding(CONTENT_PADDING))
                        .width(Length::Fill)
                        .height(Length::Fill),
                )
                .push(rule::horizontal(1))
                .push(
                    container(
                        Row::new()
                            .spacing(8)
                            .align_y(alignment::Vertical::Center)
                            .push(edit_json)
                            .push(iced::widget::Space::new().width(Length::Fill))
                            .push(save),
                    )
                    .padding([16.0, CONTENT_PADDING]),
                ),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |_: &Theme| container::Style {
            background: Some(Background::Color(self.tokens.colors.background)),
            text_color: Some(self.tokens.colors.background_fg),
            ..Default::default()
        })
        .into()
    }

    fn section_title<'a>(&'a self, title: &'a str) -> Element<'a, SettingsMessage> {
        text(title)
            .size(13)
            .color(self.tokens.colors.primary)
            .into()
    }

    fn setting_row<'a>(
        &'a self,
        title: &'a str,
        description: &'a str,
        control: Element<'a, SettingsMessage>,
    ) -> Element<'a, SettingsMessage> {
        Row::new()
            .spacing(20)
            .align_y(alignment::Vertical::Center)
            .push(
                Column::new()
                    .spacing(2)
                    .width(Length::Fill)
                    .push(
                        text(title)
                            .size(14)
                            .width(Length::Fill)
                            .wrapping(iced::widget::text::Wrapping::None)
                            .color(self.tokens.colors.card_fg),
                    )
                    .push(
                        text(description)
                            .size(12)
                            .width(Length::Fill)
                            .color(self.tokens.colors.muted_fg),
                    ),
            )
            .push(control)
            .into()
    }

    fn number_input<'a>(
        &'a self,
        value: &'a str,
        valid: bool,
        on_input: fn(String) -> SettingsMessage,
    ) -> Element<'a, SettingsMessage> {
        text_input("", value)
            .on_input(on_input)
            .width(CONTROL_WIDTH)
            .padding([CONTROL_PADDING, 10.0])
            .size(14)
            .align_x(alignment::Horizontal::Right)
            .style(move |_, status| style::text_input(self.tokens, status, !valid))
            .into()
    }
}
