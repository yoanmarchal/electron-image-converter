// Système de design : couleurs, espacements et styles de chaque contrôle, en clair et en sombre
use iced::border::{self, Border};
use iced::font::{self, Font};
use iced::theme::Palette;
use iced::widget::overlay::menu;
use iced::widget::{
    button, checkbox, container, pick_list, progress_bar, scrollable, slider, text_input, toggler,
};
use iced::{Background, Color, Shadow, Theme, Vector, color};

pub const SPACE_XS: f32 = 4.0;
pub const SPACE_SM: f32 = 8.0;
pub const SPACE_MD: f32 = 12.0;
pub const SPACE_LG: f32 = 16.0;
pub const SPACE_XL: f32 = 24.0;

pub const RADIUS_SM: f32 = 4.0;
pub const RADIUS_MD: f32 = 6.0;
pub const RADIUS_LG: f32 = 8.0;

pub const TEXT_XS: f32 = 12.0;
pub const TEXT_SM: f32 = 14.0;
pub const TEXT_LG: f32 = 18.0;
pub const TEXT_XL: f32 = 20.0;

pub const ICON_MD: f32 = 16.0;

pub const FONT: Font = Font::with_name("Inter");
pub const FONT_MEDIUM: Font = Font {
    weight: font::Weight::Medium,
    ..FONT
};
pub const FONT_SEMIBOLD: Font = Font {
    weight: font::Weight::Semibold,
    ..FONT
};

#[derive(Debug, Clone, Copy)]
pub struct Tokens {
    pub background: Color,
    pub surface: Color,
    /// Fond des zones en retrait : champs en lecture seule, en-têtes de tableau, pistes.
    pub surface_muted: Color,
    pub surface_hover: Color,
    pub border: Color,
    pub border_strong: Color,
    pub text: Color,
    pub text_secondary: Color,
    pub text_muted: Color,
    pub primary: Color,
    pub primary_hover: Color,
    /// Texte et icônes à la couleur d'accent, lisibles sur `surface`.
    pub primary_text: Color,
    pub primary_soft: Color,
    pub primary_soft_text: Color,
    pub on_primary: Color,
    pub danger: Color,
    pub danger_hover: Color,
    pub danger_text: Color,
    pub danger_soft: Color,
    pub success_text: Color,
    pub success_soft: Color,
    pub warning: Color,
    pub shadow: Color,
}

// Mêmes teintes que l'ancienne interface (palette Tailwind : gray, teal, red, green)
pub const LIGHT: Tokens = Tokens {
    background: color!(0xf9fafb),
    surface: color!(0xffffff),
    surface_muted: color!(0xf3f4f6),
    surface_hover: color!(0xe5e7eb),
    border: color!(0xe5e7eb),
    border_strong: color!(0xd1d5db),
    text: color!(0x111827),
    text_secondary: color!(0x374151),
    text_muted: color!(0x6b7280),
    primary: color!(0x0d9488),
    primary_hover: color!(0x0f766e),
    primary_text: color!(0x0d9488),
    primary_soft: color!(0xccfbf1),
    primary_soft_text: color!(0x0f766e),
    on_primary: color!(0xffffff),
    danger: color!(0xdc2626),
    danger_hover: color!(0xb91c1c),
    danger_text: color!(0xdc2626),
    danger_soft: color!(0xfee2e2),
    success_text: color!(0x16a34a),
    success_soft: color!(0xdcfce7),
    warning: color!(0xca8a04),
    shadow: color!(0x000000, 0.10),
};

pub const DARK: Tokens = Tokens {
    background: color!(0x111827),
    surface: color!(0x1f2937),
    surface_muted: color!(0x111827),
    surface_hover: color!(0x374151),
    border: color!(0x374151),
    border_strong: color!(0x4b5563),
    text: color!(0xf9fafb),
    text_secondary: color!(0xd1d5db),
    text_muted: color!(0x9ca3af),
    primary: color!(0x0d9488),
    primary_hover: color!(0x14b8a6),
    primary_text: color!(0x2dd4bf),
    primary_soft: color!(0x134e4a),
    primary_soft_text: color!(0xccfbf1),
    on_primary: color!(0xffffff),
    danger: color!(0xdc2626),
    danger_hover: color!(0xef4444),
    danger_text: color!(0xf87171),
    danger_soft: color!(0x7f1d1d),
    success_text: color!(0x4ade80),
    success_soft: color!(0x14532d),
    warning: color!(0xeab308),
    shadow: color!(0x000000, 0.35),
};

pub fn tokens(dark_mode: bool) -> Tokens {
    if dark_mode { DARK } else { LIGHT }
}

/// Thème Iced de base : sert aux rares éléments qui n'ont pas de style explicite.
pub fn iced_theme(dark_mode: bool) -> Theme {
    let tokens = tokens(dark_mode);
    Theme::custom(
        if dark_mode { "Sombre" } else { "Clair" },
        Palette {
            background: tokens.background,
            text: tokens.text,
            primary: tokens.primary,
            success: tokens.success_text,
            warning: tokens.warning,
            danger: tokens.danger,
        },
    )
}

fn with_alpha(color: Color, alpha: f32) -> Color {
    Color { a: alpha, ..color }
}

fn outline(color: Color, radius: f32) -> Border {
    Border {
        color,
        width: 1.0,
        radius: radius.into(),
    }
}

// --- Conteneurs ---

pub fn page(tokens: Tokens) -> container::Style {
    container::Style {
        background: Some(tokens.background.into()),
        text_color: Some(tokens.text),
        ..container::Style::default()
    }
}

pub fn card(tokens: Tokens) -> container::Style {
    container::Style {
        background: Some(tokens.surface.into()),
        border: outline(tokens.border, RADIUS_LG),
        shadow: Shadow {
            color: tokens.shadow,
            offset: Vector::new(0.0, 2.0),
            blur_radius: 6.0,
        },
        ..container::Style::default()
    }
}

pub fn header(tokens: Tokens) -> container::Style {
    container::Style {
        background: Some(tokens.surface.into()),
        shadow: Shadow {
            color: tokens.shadow,
            offset: Vector::new(0.0, 1.0),
            blur_radius: 3.0,
        },
        ..container::Style::default()
    }
}

pub fn status_bar(tokens: Tokens) -> container::Style {
    container::Style {
        background: Some(tokens.surface_muted.into()),
        text_color: Some(tokens.text_muted),
        ..container::Style::default()
    }
}

pub fn separator(tokens: Tokens) -> container::Style {
    container::Style {
        background: Some(tokens.border.into()),
        ..container::Style::default()
    }
}

pub fn drop_zone(tokens: Tokens, is_hovered: bool) -> container::Style {
    container::Style {
        background: Some(
            if is_hovered {
                with_alpha(tokens.primary_soft, 0.5)
            } else {
                tokens.surface
            }
            .into(),
        ),
        border: Border {
            color: if is_hovered {
                tokens.primary
            } else {
                tokens.border_strong
            },
            width: 2.0,
            radius: RADIUS_LG.into(),
        },
        ..container::Style::default()
    }
}

/// Pastille ronde derrière une grande icône.
pub fn icon_disc(tokens: Tokens) -> container::Style {
    container::Style {
        background: Some(tokens.primary_soft.into()),
        border: border::rounded(999),
        ..container::Style::default()
    }
}

pub fn list_row(tokens: Tokens) -> container::Style {
    container::Style {
        background: Some(tokens.surface.into()),
        border: outline(tokens.border, RADIUS_MD),
        ..container::Style::default()
    }
}

pub fn thumbnail(tokens: Tokens) -> container::Style {
    container::Style {
        background: Some(tokens.surface_muted.into()),
        border: border::rounded(RADIUS_SM),
        ..container::Style::default()
    }
}

/// Champ en lecture seule (dossier de sortie).
pub fn read_only_field(tokens: Tokens) -> container::Style {
    container::Style {
        background: Some(tokens.surface_muted.into()),
        text_color: Some(tokens.text_secondary),
        border: outline(tokens.border_strong, RADIUS_MD),
        ..container::Style::default()
    }
}

pub fn table_header(tokens: Tokens) -> container::Style {
    container::Style {
        background: Some(tokens.surface_muted.into()),
        text_color: Some(tokens.text_muted),
        border: Border {
            radius: border::top(RADIUS_LG),
            ..Border::default()
        },
        ..container::Style::default()
    }
}

pub fn badge(tokens: Tokens, is_positive: bool) -> container::Style {
    let (background, text) = if is_positive {
        (tokens.success_soft, tokens.success_text)
    } else {
        (tokens.danger_soft, tokens.danger_text)
    };
    container::Style {
        background: Some(background.into()),
        text_color: Some(text),
        border: border::rounded(999),
        ..container::Style::default()
    }
}

pub fn tooltip(tokens: Tokens) -> container::Style {
    container::Style {
        background: Some(tokens.text.into()),
        text_color: Some(tokens.surface),
        border: border::rounded(RADIUS_SM),
        ..container::Style::default()
    }
}

/// Voile posé sur les réglages pendant une conversion.
pub fn veil(tokens: Tokens) -> container::Style {
    container::Style {
        background: Some(with_alpha(tokens.surface, 0.6).into()),
        border: border::rounded(RADIUS_LG),
        ..container::Style::default()
    }
}

// --- Boutons ---

fn filled_button(
    tokens: Tokens,
    status: button::Status,
    color: Color,
    hover: Color,
) -> button::Style {
    let background = match status {
        button::Status::Active => color,
        button::Status::Hovered | button::Status::Pressed => hover,
        button::Status::Disabled => with_alpha(color, 0.45),
    };
    button::Style {
        background: Some(background.into()),
        text_color: tokens.on_primary,
        border: border::rounded(RADIUS_MD),
        ..button::Style::default()
    }
}

pub fn primary_button(tokens: Tokens, status: button::Status) -> button::Style {
    filled_button(tokens, status, tokens.primary, tokens.primary_hover)
}

pub fn danger_button(tokens: Tokens, status: button::Status) -> button::Style {
    filled_button(tokens, status, tokens.danger, tokens.danger_hover)
}

/// Bouton à bordure, pour les actions secondaires.
pub fn outline_button(tokens: Tokens, status: button::Status) -> button::Style {
    let background = match status {
        button::Status::Hovered | button::Status::Pressed => tokens.surface_hover,
        _ => tokens.surface_muted,
    };
    button::Style {
        background: Some(background.into()),
        text_color: match status {
            button::Status::Disabled => tokens.text_muted,
            _ => tokens.text_secondary,
        },
        border: outline(tokens.border_strong, RADIUS_MD),
        ..button::Style::default()
    }
}

/// Onglet de navigation de l'en-tête.
pub fn tab_button(tokens: Tokens, is_active: bool, status: button::Status) -> button::Style {
    let (background, text_color) = match (is_active, status) {
        (true, _) => (Some(tokens.primary_soft), tokens.primary_soft_text),
        (false, button::Status::Hovered | button::Status::Pressed) => {
            (Some(tokens.surface_muted), tokens.text_secondary)
        }
        (false, _) => (None, tokens.text_muted),
    };
    button::Style {
        background: background.map(Background::from),
        text_color,
        border: border::rounded(RADIUS_MD),
        ..button::Style::default()
    }
}

/// Bouton sans fond dont seul le texte change de couleur au survol (icône, lien).
pub fn text_button(status: button::Status, color: Color, hover: Color) -> button::Style {
    button::Style {
        background: None,
        text_color: match status {
            button::Status::Hovered | button::Status::Pressed => hover,
            button::Status::Disabled => with_alpha(color, 0.45),
            button::Status::Active => color,
        },
        ..button::Style::default()
    }
}

pub fn link_button(tokens: Tokens, status: button::Status) -> button::Style {
    text_button(status, tokens.primary_text, tokens.primary_hover)
}

pub fn danger_link_button(tokens: Tokens, status: button::Status) -> button::Style {
    text_button(status, tokens.danger_text, tokens.danger_hover)
}

/// Bouton discret qui vire au rouge au survol (retirer une image).
pub fn remove_button(tokens: Tokens, status: button::Status) -> button::Style {
    text_button(status, tokens.text_muted, tokens.danger_text)
}

pub fn icon_button(tokens: Tokens, status: button::Status) -> button::Style {
    button::Style {
        background: match status {
            button::Status::Hovered | button::Status::Pressed => Some(tokens.surface_muted.into()),
            _ => None,
        },
        text_color: match status {
            button::Status::Hovered | button::Status::Pressed => tokens.text,
            _ => tokens.text_muted,
        },
        border: border::rounded(999),
        ..button::Style::default()
    }
}

// --- Champs ---

pub fn pick_list_style(tokens: Tokens, status: pick_list::Status) -> pick_list::Style {
    pick_list::Style {
        text_color: tokens.text,
        placeholder_color: tokens.text_muted,
        handle_color: tokens.text_muted,
        background: tokens.surface.into(),
        border: outline(
            match status {
                pick_list::Status::Active => tokens.border_strong,
                pick_list::Status::Hovered | pick_list::Status::Opened { .. } => tokens.primary,
            },
            RADIUS_MD,
        ),
    }
}

pub fn menu(tokens: Tokens) -> menu::Style {
    menu::Style {
        background: tokens.surface.into(),
        border: outline(tokens.border_strong, RADIUS_MD),
        text_color: tokens.text,
        selected_text_color: tokens.primary_soft_text,
        selected_background: tokens.primary_soft.into(),
        shadow: Shadow {
            color: tokens.shadow,
            offset: Vector::new(0.0, 4.0),
            blur_radius: 12.0,
        },
    }
}

pub fn text_input_style(
    tokens: Tokens,
    status: text_input::Status,
    is_valid: bool,
) -> text_input::Style {
    let border_color = match status {
        _ if !is_valid => tokens.danger_text,
        text_input::Status::Focused { .. } | text_input::Status::Hovered => tokens.primary,
        _ => tokens.border_strong,
    };
    text_input::Style {
        background: tokens.surface.into(),
        border: Border {
            width: if matches!(status, text_input::Status::Focused { .. }) {
                2.0
            } else {
                1.0
            },
            ..outline(border_color, RADIUS_MD)
        },
        icon: tokens.text_muted,
        placeholder: tokens.text_muted,
        value: tokens.text,
        selection: with_alpha(tokens.primary, 0.35),
    }
}

pub fn slider_style(tokens: Tokens, status: slider::Status, is_enabled: bool) -> slider::Style {
    let accent = match status {
        _ if !is_enabled => tokens.border_strong,
        slider::Status::Active => tokens.primary,
        slider::Status::Hovered | slider::Status::Dragged => tokens.primary_hover,
    };
    slider::Style {
        rail: slider::Rail {
            backgrounds: (accent.into(), tokens.surface_hover.into()),
            width: 6.0,
            border: border::rounded(999),
        },
        handle: slider::Handle {
            shape: slider::HandleShape::Circle { radius: 8.0 },
            background: accent.into(),
            border_width: 2.0,
            border_color: tokens.surface,
        },
    }
}

pub fn checkbox_style(tokens: Tokens, status: checkbox::Status) -> checkbox::Style {
    let (is_checked, is_hovered) = match status {
        checkbox::Status::Active { is_checked } | checkbox::Status::Disabled { is_checked } => {
            (is_checked, false)
        }
        checkbox::Status::Hovered { is_checked } => (is_checked, true),
    };
    checkbox::Style {
        background: if is_checked {
            tokens.primary
        } else {
            tokens.surface
        }
        .into(),
        icon_color: tokens.on_primary,
        border: outline(
            if is_checked || is_hovered {
                tokens.primary
            } else {
                tokens.border_strong
            },
            RADIUS_SM,
        ),
        text_color: Some(tokens.text_secondary),
    }
}

pub fn toggler_style(tokens: Tokens, status: toggler::Status) -> toggler::Style {
    let is_toggled = match status {
        toggler::Status::Active { is_toggled }
        | toggler::Status::Hovered { is_toggled }
        | toggler::Status::Disabled { is_toggled } => is_toggled,
    };
    let is_hovered = matches!(status, toggler::Status::Hovered { .. });
    toggler::Style {
        background: match (is_toggled, is_hovered) {
            (true, false) => tokens.primary,
            (true, true) => tokens.primary_hover,
            (false, false) => tokens.border_strong,
            (false, true) => tokens.text_muted,
        }
        .into(),
        background_border_width: 0.0,
        background_border_color: Color::TRANSPARENT,
        foreground: tokens.on_primary.into(),
        foreground_border_width: 0.0,
        foreground_border_color: Color::TRANSPARENT,
        text_color: Some(tokens.text),
        border_radius: None,
        padding_ratio: 0.15,
    }
}

pub fn progress_bar_style(tokens: Tokens) -> progress_bar::Style {
    progress_bar::Style {
        background: tokens.surface_hover.into(),
        bar: tokens.primary.into(),
        border: border::rounded(999),
    }
}

pub fn scrollable_style(tokens: Tokens, status: scrollable::Status) -> scrollable::Style {
    let is_active = matches!(
        status,
        scrollable::Status::Hovered {
            is_vertical_scrollbar_hovered: true,
            ..
        } | scrollable::Status::Dragged { .. }
    );
    let rail = scrollable::Rail {
        background: None,
        border: Border::default(),
        scroller: scrollable::Scroller {
            background: if is_active {
                tokens.text_muted
            } else {
                tokens.border_strong
            }
            .into(),
            border: border::rounded(999),
        },
    };
    scrollable::Style {
        container: container::Style::default(),
        vertical_rail: rail,
        horizontal_rail: rail,
        gap: None,
        auto_scroll: scrollable::AutoScroll {
            background: tokens.surface.into(),
            border: outline(tokens.border_strong, 999.0),
            shadow: Shadow::default(),
            icon: tokens.text_muted,
        },
    }
}
