// Écran « Historique » : tableau des conversions passées
use chrono::Local;
use iced::widget::{button, column, container, row, scrollable, space, text};
use iced::{Center, Element, Fill, FillPortion};
use lucide_icons::iced::{icon_calendar, icon_file_text, icon_trash_2};

use super::convert::{dimensions_change, open_button};
use super::theme::{self, Tokens};
use super::widgets::{empty_state, separator};
use super::{App, Message, file_name};
use crate::format::{format_date, format_file_size, format_percent};
use crate::history::HistoryItem;

// Largeurs relatives des colonnes : fichier, tailles, dimensions, gain, date, actions
const COLUMNS: [u16; 7] = [4, 2, 2, 5, 2, 4, 2];

pub fn view(app: &App) -> Element<'_, Message> {
    let tokens = app.tokens;
    let items = app.history.items();
    if items.is_empty() {
        return empty_state(icon_file_text(), "Aucune conversion pour le moment", tokens);
    }

    // La confirmation se fait dans la page : une boîte de dialogue native n'existe pas sur
    // toutes les distributions Linux
    let actions: Element<'_, Message> = if app.is_confirming_clear {
        row![
            text("Vider tout l'historique ?")
                .size(theme::TEXT_SM)
                .color(tokens.text_secondary),
            button(text("Annuler").font(theme::FONT_MEDIUM))
                .padding([theme::SPACE_SM, theme::SPACE_LG])
                .on_press(Message::ClearHistoryConfirmed(false))
                .style(move |_, status| theme::outline_button(tokens, status)),
            button(text("Vider").font(theme::FONT_MEDIUM))
                .padding([theme::SPACE_SM, theme::SPACE_LG])
                .on_press(Message::ClearHistoryConfirmed(true))
                .style(move |_, status| theme::danger_button(tokens, status)),
        ]
        .spacing(theme::SPACE_MD)
        .align_y(Center)
        .into()
    } else {
        button(
            row![
                icon_trash_2().size(theme::ICON_MD),
                text("Vider l'historique").font(theme::FONT_MEDIUM)
            ]
            .spacing(theme::SPACE_SM)
            .align_y(Center),
        )
        .padding([theme::SPACE_SM, theme::SPACE_LG])
        .on_press(Message::ClearHistory)
        .style(move |_, status| theme::danger_button(tokens, status))
        .into()
    };

    let title = row![
        text("Historique des conversions")
            .size(theme::TEXT_XL)
            .font(theme::FONT_SEMIBOLD)
            .color(tokens.text),
        space().width(Fill),
        actions,
    ]
    .align_y(Center);

    let heading = |label: &'static str| -> Element<'_, Message> {
        text(label)
            .size(theme::TEXT_XS)
            .font(theme::FONT_MEDIUM)
            .into()
    };
    let header = container(table_row([
        heading("FICHIER"),
        heading("ORIGINE"),
        heading("CONVERTIE"),
        heading("DIMENSIONS"),
        heading("GAIN"),
        heading("DATE"),
        heading("ACTIONS"),
    ]))
    .style(move |_| theme::table_header(tokens));

    let rows = column(
        items
            .iter()
            .flat_map(|item| [separator(tokens), history_row(item, tokens)]),
    );

    column![
        title,
        container(column![
            header,
            scrollable(rows).style(move |_, status| theme::scrollable_style(tokens, status)),
        ])
        .style(move |_| theme::card(tokens)),
    ]
    .spacing(theme::SPACE_LG)
    .padding(theme::SPACE_LG)
    .into()
}

fn table_row<'a>(cells: [Element<'a, Message>; 7]) -> Element<'a, Message> {
    row(cells.into_iter().zip(COLUMNS).map(|(cell, portion)| {
        container(cell)
            .width(FillPortion(portion))
            .clip(true)
            .into()
    }))
    .spacing(theme::SPACE_MD)
    .padding([theme::SPACE_MD, theme::SPACE_LG])
    .align_y(Center)
    .into()
}

fn history_row(item: &HistoryItem, tokens: Tokens) -> Element<'_, Message> {
    let cell = |content: String| -> Element<'_, Message> {
        text(content)
            .size(theme::TEXT_SM)
            .color(tokens.text_muted)
            .wrapping(text::Wrapping::None)
            .into()
    };

    let file = column![
        text(file_name(&item.original_path))
            .size(theme::TEXT_SM)
            .font(theme::FONT_MEDIUM)
            .color(tokens.text)
            .wrapping(text::Wrapping::None),
        text(format!("→ {}", file_name(&item.output_path)))
            .size(theme::TEXT_XS)
            .color(tokens.text_muted)
            .wrapping(text::Wrapping::None),
    ]
    .spacing(2);

    let is_smaller = item.compression_ratio >= 0.0;
    let gain = container(
        text(format_percent(item.compression_ratio))
            .size(theme::TEXT_XS)
            .font(theme::FONT_SEMIBOLD),
    )
    .padding([2.0, theme::SPACE_SM])
    .style(move |_| theme::badge(tokens, is_smaller));

    let date = row![
        icon_calendar()
            .size(theme::TEXT_XS)
            .color(tokens.text_muted),
        text(format_date(
            item.timestamp.with_timezone(&Local).naive_local()
        ))
        .size(theme::TEXT_SM)
        .color(tokens.text_muted)
        .wrapping(text::Wrapping::None),
    ]
    .spacing(theme::SPACE_XS)
    .align_y(Center);

    table_row([
        file.into(),
        cell(format_file_size(item.original_size)),
        cell(format_file_size(item.new_size)),
        cell(dimensions_change(
            item.original_dimensions,
            item.new_dimensions,
        )),
        gain.into(),
        date.into(),
        open_button(&item.output_path, theme::TEXT_SM, tokens),
    ])
}
