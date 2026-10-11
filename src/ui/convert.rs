// Écran « Convertir » : zone de dépôt, liste des images et panneau de réglages
use iced::widget::{
    button, checkbox, column, container, mouse_area, opaque, pick_list, progress_bar, row,
    scrollable, slider, space, stack, text, text_input, toggler, tooltip,
};
use iced::{Center, Element, Fill, FillPortion, mouse};
use lucide_icons::iced::{
    icon_external_link, icon_file_check, icon_file_x, icon_folder, icon_image_down, icon_play,
    icon_refresh_cw, icon_scaling, icon_sliders_horizontal, icon_trash_2, icon_undo_2, icon_upload,
};

use super::theme::{self, Tokens};
use super::widgets::{empty_state, separator, with_tooltip};
use super::{App, FORMAT_CHOICES, ImageEntry, Message, RESIZE_MODES, Status};
use crate::conversion::{Resize, ResizeMode, target_dimensions};
use crate::format::{format_dimensions, format_file_size, format_percent};

pub fn view(app: &App) -> Element<'_, Message> {
    row![
        column![drop_zone(app), image_list(app)]
            .spacing(theme::SPACE_LG)
            .width(FillPortion(2)),
        container(scrollable(settings_panel(app)).style({
            let tokens = app.tokens;
            move |_, status| theme::scrollable_style(tokens, status)
        }))
        .width(FillPortion(1)),
    ]
    .spacing(theme::SPACE_LG)
    .padding(theme::SPACE_LG)
    .into()
}

fn drop_zone(app: &App) -> Element<'_, Message> {
    let tokens = app.tokens;
    let is_enabled = !app.is_converting;

    let title = if app.loading > 0 {
        "Chargement des images…"
    } else if app.is_hovered {
        "Déposez les images ici"
    } else {
        "Glissez-déposez vos images ici"
    };

    let content = column![
        container(icon_upload().size(36).color(tokens.primary_text))
            .center(68)
            .style(move |_| theme::icon_disc(tokens)),
        space().height(theme::SPACE_XS),
        text(title)
            .size(theme::TEXT_LG)
            .font(theme::FONT_MEDIUM)
            .color(tokens.text_secondary),
        text("ou cliquez pour choisir des fichiers (JPEG, PNG, GIF, TIFF, WebP)")
            .size(theme::TEXT_SM)
            .color(tokens.text_muted),
        space().height(theme::SPACE_XS),
        button(
            row![
                icon_image_down().size(theme::ICON_MD),
                text("Choisir des images").font(theme::FONT_MEDIUM)
            ]
            .spacing(theme::SPACE_SM)
            .align_y(Center),
        )
        .padding([theme::SPACE_SM, theme::SPACE_LG])
        .on_press_maybe(is_enabled.then_some(Message::SelectFiles))
        .style(move |_, status| theme::primary_button(tokens, status)),
    ]
    .spacing(theme::SPACE_SM)
    .align_x(Center);

    let is_hovered = app.is_hovered;
    let zone = container(content)
        .center_x(Fill)
        .padding([theme::SPACE_XL, theme::SPACE_LG])
        .style(move |_| theme::drop_zone(tokens, is_hovered));

    if is_enabled {
        mouse_area(zone)
            .on_press(Message::SelectFiles)
            .interaction(mouse::Interaction::Pointer)
            .into()
    } else {
        zone.into()
    }
}

fn image_list(app: &App) -> Element<'_, Message> {
    let tokens = app.tokens;
    if app.images.is_empty() {
        return empty_state(icon_file_x(), "Aucune image ajoutée", tokens);
    }
    // Une saisie de taille invalide n'affiche pas d'aperçu des dimensions
    let resize = app.settings.resize().ok().flatten();

    let title = row![
        text(format!("Images ({})", app.images.len()))
            .font(theme::FONT_MEDIUM)
            .color(tokens.text_secondary),
        space().width(Fill),
        button(
            row![
                icon_trash_2().size(theme::TEXT_XS),
                text("Tout retirer").size(theme::TEXT_SM)
            ]
            .spacing(theme::SPACE_XS)
            .align_y(Center),
        )
        .padding(0)
        .on_press(Message::RemoveAllImages)
        .style(move |_, status| theme::danger_link_button(tokens, status)),
    ]
    .align_y(Center);

    let rows = column(
        app.images
            .iter()
            .map(|image| image_row(image, resize.as_ref(), tokens)),
    )
    .spacing(theme::SPACE_SM)
    // Garde la barre de défilement à l'écart des lignes
    .padding(iced::Padding::ZERO.right(theme::SPACE_MD));

    container(
        column![
            title,
            separator(tokens),
            scrollable(rows)
                .height(Fill)
                .style(move |_, status| theme::scrollable_style(tokens, status)),
        ]
        .spacing(theme::SPACE_SM),
    )
    .padding(theme::SPACE_LG)
    .height(Fill)
    .style(move |_| theme::card(tokens))
    .into()
}

fn image_row<'a>(
    image: &'a ImageEntry,
    resize: Option<&Resize>,
    tokens: Tokens,
) -> Element<'a, Message> {
    let thumbnail = container(
        iced::widget::image(image.thumbnail.clone())
            .width(48)
            .height(48)
            .border_radius(theme::RADIUS_SM),
    )
    .style(move |_| theme::thumbnail(tokens));

    let status_icon: Option<Element<'_, Message>> = match &image.status {
        Status::Pending => None,
        Status::Converting => Some(with_tooltip(
            icon_refresh_cw().size(theme::ICON_MD).color(tokens.warning),
            "Conversion en cours",
            tokens,
        )),
        Status::Converted(_) => Some(with_tooltip(
            icon_file_check()
                .size(theme::ICON_MD)
                .color(tokens.success_text),
            "Convertie",
            tokens,
        )),
        Status::Failed(_) => Some(with_tooltip(
            icon_file_x().size(theme::ICON_MD).color(tokens.danger_text),
            "Erreur",
            tokens,
        )),
    };

    let name = row![
        container(
            text(&image.name)
                .size(theme::TEXT_SM)
                .font(theme::FONT_MEDIUM)
                .color(tokens.text)
                .wrapping(text::Wrapping::None),
        )
        .width(Fill)
        .clip(true),
    ]
    .push(status_icon)
    .push(with_tooltip(
        button(icon_trash_2().size(theme::ICON_MD))
            .padding(0)
            .on_press(Message::RemoveImage(image.id))
            .style(move |_, status| theme::remove_button(tokens, status)),
        "Retirer de la liste",
        tokens,
    ))
    .spacing(theme::SPACE_SM)
    .align_y(Center);

    let detail = |content: String| text(content).size(theme::TEXT_XS).color(tokens.text_muted);
    let mut details = row![detail(format_file_size(image.size))]
        .spacing(theme::SPACE_MD)
        .align_y(Center);

    match &image.status {
        Status::Pending | Status::Converting => {
            details = details.push(dimensions(image.dimensions, resize, tokens));
        }
        Status::Converted(result) => {
            let is_smaller = result.compression_ratio >= 0.0;
            details = details
                .push(detail(format!("→ {}", format_file_size(result.new_size))))
                .push(detail(dimensions_change(
                    result.original_dimensions,
                    result.new_dimensions,
                )))
                .push(
                    text(format!(
                        "Gain : {}",
                        format_percent(result.compression_ratio)
                    ))
                    .size(theme::TEXT_XS)
                    .color(if is_smaller {
                        tokens.success_text
                    } else {
                        tokens.danger_text
                    }),
                )
                .push(open_button(&result.output_path, theme::TEXT_XS, tokens));
        }
        Status::Failed(error) => {
            details = details.push(text(error).size(theme::TEXT_XS).color(tokens.danger_text));
        }
    }

    container(
        row![
            thumbnail,
            column![name, details.wrap()].spacing(theme::SPACE_XS)
        ]
        .spacing(theme::SPACE_MD)
        .align_y(Center),
    )
    .padding(theme::SPACE_SM)
    .style(move |_| theme::list_row(tokens))
    .into()
}

/// Dimensions de l'image, suivies de celles qu'elle aura après redimensionnement.
fn dimensions<'a>(
    original: (u32, u32),
    resize: Option<&Resize>,
    tokens: Tokens,
) -> Element<'a, Message> {
    let target = resize
        .map(|resize| target_dimensions(original.0, original.1, resize))
        .filter(|target| *target != original);

    let mut line = row![
        text(format_dimensions(original))
            .size(theme::TEXT_XS)
            .color(tokens.text_muted)
    ]
    .spacing(theme::SPACE_XS);
    if let Some(target) = target {
        line = line.push(
            text(format!("→ {}", format_dimensions(target)))
                .size(theme::TEXT_XS)
                .color(tokens.primary_text),
        );
    }
    line.into()
}

pub fn dimensions_change(original: (u32, u32), new: (u32, u32)) -> String {
    if original == new {
        format_dimensions(original)
    } else {
        format!(
            "{} → {}",
            format_dimensions(original),
            format_dimensions(new)
        )
    }
}

pub fn open_button<'a>(path: &std::path::Path, size: f32, tokens: Tokens) -> Element<'a, Message> {
    button(
        row![icon_external_link().size(size), text("Ouvrir").size(size)]
            .spacing(theme::SPACE_XS)
            .align_y(Center),
    )
    .padding(0)
    .on_press(Message::OpenFile(path.to_owned()))
    .style(move |_, status| theme::link_button(tokens, status))
    .into()
}

fn field_label<'a>(label: impl text::IntoFragment<'a>, tokens: Tokens) -> Element<'a, Message> {
    text(label)
        .size(theme::TEXT_SM)
        .font(theme::FONT_MEDIUM)
        .color(tokens.text_secondary)
        .into()
}

fn hint<'a>(label: impl text::IntoFragment<'a>, tokens: Tokens) -> Element<'a, Message> {
    text(label)
        .size(theme::TEXT_XS)
        .color(tokens.text_muted)
        .into()
}

fn settings_panel(app: &App) -> Element<'_, Message> {
    let tokens = app.tokens;

    let title = row![
        icon_sliders_horizontal()
            .size(20)
            .color(tokens.primary_text),
        text("Paramètres de conversion")
            .size(theme::TEXT_LG)
            .font(theme::FONT_SEMIBOLD)
            .color(tokens.text),
    ]
    .spacing(theme::SPACE_SM)
    .align_y(Center);

    let controls = column![
        format_field(app),
        quality_field(app),
        resize_field(app),
        output_dir_field(app),
    ]
    .spacing(theme::SPACE_LG);

    // Pendant une conversion, un voile bloque les réglages : ils valent pour tout le lot
    let controls: Element<'_, Message> = if app.is_converting {
        stack![
            controls,
            opaque(
                container(space())
                    .width(Fill)
                    .height(Fill)
                    .style(move |_| theme::veil(tokens))
            ),
        ]
        .into()
    } else {
        controls.into()
    };

    container(
        column![title, controls, separator(tokens), convert_action(app)].spacing(theme::SPACE_LG),
    )
    .padding(theme::SPACE_LG)
    .style(move |_| theme::card(tokens))
    .into()
}

fn format_field(app: &App) -> Element<'_, Message> {
    let tokens = app.tokens;
    column![
        field_label("Format de sortie", tokens),
        pick_list(
            &FORMAT_CHOICES[..],
            Some(app.settings.format),
            Message::FormatSelected
        )
        .width(Fill)
        .padding([theme::SPACE_SM, theme::SPACE_MD])
        .text_size(theme::TEXT_SM)
        .style(move |_, status| theme::pick_list_style(tokens, status))
        .menu_style(move |_| theme::menu(tokens)),
    ]
    .spacing(theme::SPACE_XS)
    .into()
}

fn quality_field(app: &App) -> Element<'_, Message> {
    let tokens = app.tokens;
    let quality = app.settings.quality;
    let is_lossless = app.settings.is_lossless();

    let quality_hint = if is_lossless {
        "Le PNG est sans perte : la qualité ne s'applique pas"
    } else if quality < 30 {
        "Qualité basse, fichier plus petit"
    } else if quality < 70 {
        "Qualité équilibrée"
    } else {
        "Haute qualité, fichier plus grand"
    };

    column![
        field_label(format!("Qualité : {quality} %"), tokens),
        row![
            hint("Basse", tokens),
            slider(1..=100, quality, Message::QualityChanged)
                .style(move |_, status| theme::slider_style(tokens, status, !is_lossless)),
            hint("Haute", tokens),
        ]
        .spacing(theme::SPACE_SM)
        .align_y(Center),
        hint(quality_hint, tokens),
    ]
    .spacing(theme::SPACE_XS)
    .into()
}

fn resize_field(app: &App) -> Element<'_, Message> {
    let tokens = app.tokens;
    let settings = &app.settings;

    let toggle = row![
        icon_scaling().size(theme::ICON_MD).color(tokens.text_muted),
        text("Redimensionner")
            .size(theme::TEXT_SM)
            .font(theme::FONT_MEDIUM)
            .color(tokens.text_secondary),
        space().width(Fill),
        toggler(settings.resize_enabled)
            .on_toggle(Message::ResizeToggled)
            .size(20)
            .style(move |_, status| theme::toggler_style(tokens, status)),
    ]
    .spacing(theme::SPACE_SM)
    .align_y(Center);

    if !settings.resize_enabled {
        return toggle.into();
    }

    let error = settings.resize().err();
    let is_valid = error.is_none();
    let unit = if settings.resize_mode == ResizeMode::Percent {
        "%"
    } else {
        "px"
    };

    column![
        toggle,
        pick_list(
            &RESIZE_MODES[..],
            Some(settings.resize_mode),
            Message::ResizeModeSelected
        )
        .width(Fill)
        .padding([theme::SPACE_SM, theme::SPACE_MD])
        .text_size(theme::TEXT_SM)
        .style(move |_, status| theme::pick_list_style(tokens, status))
        .menu_style(move |_| theme::menu(tokens)),
        row![
            text_input("Taille", &settings.resize_value)
                .on_input(Message::ResizeValueChanged)
                .padding([theme::SPACE_SM, theme::SPACE_MD])
                .size(theme::TEXT_SM)
                .style(move |_, status| theme::text_input_style(tokens, status, is_valid)),
            text(unit).size(theme::TEXT_SM).color(tokens.text_muted),
        ]
        .spacing(theme::SPACE_SM)
        .align_y(Center),
        match error {
            Some(error) => text(error)
                .size(theme::TEXT_XS)
                .color(tokens.danger_text)
                .into(),
            None => hint("Les proportions de l'image sont conservées", tokens),
        },
        checkbox(!settings.allow_upscale)
            .label("Ne pas agrandir les images plus petites")
            .on_toggle(|keep_small| Message::AllowUpscaleToggled(!keep_small))
            .size(16)
            .text_size(theme::TEXT_SM)
            .spacing(theme::SPACE_SM)
            .style(move |_, status| theme::checkbox_style(tokens, status)),
    ]
    .spacing(theme::SPACE_SM)
    .into()
}

fn output_dir_field(app: &App) -> Element<'_, Message> {
    let tokens = app.tokens;
    let output_dir = app.settings.output_dir.as_deref();

    let path = container(
        text(output_dir.map_or_else(
            || "Même dossier que l'original".to_owned(),
            |dir| dir.display().to_string(),
        ))
        .size(theme::TEXT_SM)
        .wrapping(text::Wrapping::None),
    )
    .padding([theme::SPACE_SM, theme::SPACE_MD])
    .width(Fill)
    .clip(true)
    .style(move |_| theme::read_only_field(tokens));

    let icon_action = |icon, message, label| {
        with_tooltip(
            button(icon)
                .padding(theme::SPACE_SM + 1.0)
                .on_press(message)
                .style(move |_, status| theme::outline_button(tokens, status)),
            label,
            tokens,
        )
    };

    // Un chemin long est coupé : l'infobulle le donne en entier
    let path: Element<'_, Message> = match output_dir {
        Some(dir) => tooltip(
            path,
            container(text(dir.display().to_string()).size(theme::TEXT_XS))
                .padding([theme::SPACE_XS, theme::SPACE_SM])
                .style(move |_| theme::tooltip(tokens)),
            tooltip::Position::Top,
        )
        .gap(theme::SPACE_XS)
        .into(),
        None => path.into(),
    };

    let mut field = row![
        path,
        icon_action(
            icon_folder().size(theme::ICON_MD),
            Message::SelectOutputDir,
            if output_dir.is_some() {
                "Changer le dossier de sortie"
            } else {
                "Choisir un dossier de sortie"
            },
        ),
    ]
    .spacing(theme::SPACE_SM)
    .align_y(Center);
    if output_dir.is_some() {
        field = field.push(icon_action(
            icon_undo_2().size(theme::ICON_MD),
            Message::OutputDirReset,
            "Revenir au dossier d'origine",
        ));
    }

    column![field_label("Dossier de sortie (optionnel)", tokens), field]
        .spacing(theme::SPACE_XS)
        .into()
}

fn convert_action(app: &App) -> Element<'_, Message> {
    let tokens = app.tokens;
    let can_convert = !app.is_converting
        && app.loading == 0
        && app.pending_count() > 0
        && app.settings.options().is_some();

    let label = if app.is_converting {
        row![text("Conversion en cours…").font(theme::FONT_MEDIUM)]
    } else {
        row![
            icon_play().size(theme::ICON_MD),
            text("Convertir les images").font(theme::FONT_MEDIUM)
        ]
    };
    let convert = button(container(label.spacing(theme::SPACE_SM).align_y(Center)).center_x(Fill))
        .padding([theme::SPACE_SM + 2.0, theme::SPACE_LG])
        .width(Fill)
        .on_press_maybe(can_convert.then_some(Message::Convert))
        .style(move |_, status| theme::primary_button(tokens, status));

    let mut action = column![convert].spacing(theme::SPACE_SM);
    if app.is_converting {
        let (done, total) = app.batch;
        action = action.push(
            progress_bar(0.0..=total as f32, done as f32)
                .girth(6)
                .style(move |_| theme::progress_bar_style(tokens)),
        );
    } else if app.loading > 0 {
        action = action.push(hint("Chargement des images…", tokens));
    } else if app.images.is_empty() {
        action = action.push(hint("Veuillez ajouter des images à convertir", tokens));
    } else if app.pending_count() == 0 {
        action = action.push(hint(
            "Toutes les images sont traitées : changez un réglage pour les reconvertir",
            tokens,
        ));
    }
    action.into()
}
