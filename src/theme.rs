//! CS2 Panorama-inspired palette + fonts (recreated; no Valve assets).
//! Bundled font: Barlow Condensed (OFL) — condensed sans similar in feel to Stratum2.
use egui::{Color32, FontData, FontDefinitions, FontFamily, FontId, Rounding, Stroke, Style, TextStyle, Visuals};

pub const BG: Color32 = Color32::from_rgba_premultiplied(14, 14, 14, 200); // #121212
pub const PANEL: Color32 = Color32::from_rgba_premultiplied(20, 20, 20, 190); // #1C1C1C
pub const PANEL_ALT: Color32 = Color32::from_rgba_premultiplied(26, 26, 26, 190);
pub const SIDEBAR: Color32 = Color32::from_rgba_premultiplied(15, 15, 15, 180);
pub const ROW: Color32 = Color32::from_rgba_premultiplied(21, 21, 21, 170); // #202020
pub const ROW_HOVER: Color32 = Color32::from_rgba_premultiplied(32, 32, 32, 200);
pub const BORDER: Color32 = Color32::from_rgb(0x3A, 0x3A, 0x3A);
pub const BORDER_SOFT: Color32 = Color32::from_rgb(0x2E, 0x2E, 0x2E);
pub const TEXT: Color32 = Color32::from_rgb(0xDC, 0xDC, 0xDC);
pub const TEXT_DIM: Color32 = Color32::from_rgb(0x8C, 0x8C, 0x8C);
pub const GOLD: Color32 = Color32::from_rgb(0xDE, 0x9B, 0x35); // #DE9B35
pub const TEAL: Color32 = Color32::from_rgb(0x4A, 0xC0, 0xC0); // CS2 settings tab accent
pub const TEAL_DIM: Color32 = Color32::from_rgb(0x2A, 0x6E, 0x6E);
pub const CENTER_PANEL: Color32 = Color32::from_rgba_premultiplied(10, 10, 10, 150);
pub const GOLD_DIM: Color32 = Color32::from_rgb(0xB4, 0x78, 0x28);
pub const GOLD_SOFT: Color32 = Color32::from_rgb(0xDE, 0x9B, 0x35);
pub const DANGER: Color32 = Color32::from_rgb(0xC8, 0x46, 0x3C);
pub const TOPBAR: Color32 = Color32::from_rgba_premultiplied(9, 9, 9, 170);
pub const TAB_STRIP: Color32 = Color32::from_rgba_premultiplied(13, 13, 13, 170);
pub const PREVIEW_BG: Color32 = Color32::from_rgb(0x3A, 0x52, 0x34);
pub const PREVIEW_GRID: Color32 = Color32::from_rgb(0x46, 0x62, 0x3E);
pub const PREVIEW_DARK: Color32 = Color32::from_rgb(0x2E, 0x42, 0x2A);

/// Preview zoom so 1px thickness / 2px length is clearly visible in the panel.
pub const CROSSHAIR_PREVIEW_ZOOM: f32 = 3.5;

pub fn install_fonts(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();
    fonts.font_data.insert(
        "barlow".into(),
        FontData::from_static(include_bytes!("../assets/fonts/BarlowCondensed-Regular.ttf")),
    );
    fonts.font_data.insert(
        "barlow_medium".into(),
        FontData::from_static(include_bytes!("../assets/fonts/BarlowCondensed-Medium.ttf")),
    );
    fonts.font_data.insert(
        "barlow_semibold".into(),
        FontData::from_static(include_bytes!("../assets/fonts/BarlowCondensed-SemiBold.ttf")),
    );
    fonts.font_data.insert(
        "barlow_bold".into(),
        FontData::from_static(include_bytes!("../assets/fonts/BarlowCondensed-Bold.ttf")),
    );

    fonts
        .families
        .entry(FontFamily::Proportional)
        .or_default()
        .insert(0, "barlow".to_owned());
    fonts
        .families
        .entry(FontFamily::Name("barlow_medium".into()))
        .or_default()
        .insert(0, "barlow_medium".to_owned());
    fonts
        .families
        .entry(FontFamily::Name("barlow_semibold".into()))
        .or_default()
        .insert(0, "barlow_semibold".to_owned());
    fonts
        .families
        .entry(FontFamily::Name("barlow_bold".into()))
        .or_default()
        .insert(0, "barlow_bold".to_owned());

    // Keep monospace fallback for share codes
    ctx.set_fonts(fonts);
}

pub fn apply(ctx: &egui::Context) {
    let mut visuals = Visuals::dark();
    visuals.window_fill = BG;
    visuals.panel_fill = PANEL;
    visuals.extreme_bg_color = SIDEBAR;
    visuals.faint_bg_color = PANEL_ALT;
    visuals.override_text_color = Some(TEXT);
    visuals.widgets.noninteractive.bg_fill = ROW;
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, BORDER_SOFT);
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, TEXT);
    visuals.widgets.inactive.bg_fill = PANEL_ALT;
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, BORDER);
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, TEXT);
    visuals.widgets.hovered.bg_fill = ROW_HOVER;
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, GOLD_DIM);
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, GOLD);
    visuals.widgets.active.bg_fill = GOLD_DIM;
    visuals.widgets.active.fg_stroke = Stroke::new(1.0, Color32::WHITE);
    visuals.widgets.open.bg_fill = PANEL_ALT;
    visuals.widgets.open.bg_stroke = Stroke::new(1.0, GOLD);
    visuals.selection.bg_fill = Color32::from_rgba_unmultiplied(0xDE, 0x9B, 0x35, 80);
    visuals.selection.stroke = Stroke::new(1.0, GOLD);
    visuals.hyperlink_color = GOLD;
    visuals.warn_fg_color = GOLD;
    visuals.error_fg_color = DANGER;
    visuals.window_stroke = Stroke::new(1.0, BORDER);
    visuals.window_rounding = Rounding::ZERO;
    visuals.menu_rounding = Rounding::ZERO;
    visuals.widgets.noninteractive.rounding = Rounding::ZERO;
    visuals.widgets.inactive.rounding = Rounding::ZERO;
    visuals.widgets.hovered.rounding = Rounding::ZERO;
    visuals.widgets.active.rounding = Rounding::ZERO;
    visuals.widgets.open.rounding = Rounding::ZERO;
    visuals.slider_trailing_fill = true;
    visuals.handle_shape = egui::style::HandleShape::Rect { aspect_ratio: 0.55 };

    let mut style = Style {
        visuals,
        ..Style::default()
    };
    style.spacing.item_spacing = egui::vec2(8.0, 6.0);
    style.spacing.button_padding = egui::vec2(16.0, 8.0);
    style.spacing.slider_width = 180.0;
    style.spacing.indent = 12.0;
    style.interaction.selectable_labels = false;

    // Condensed CS2-like type scale
    style.text_styles.insert(
        TextStyle::Heading,
        FontId::new(26.0, FontFamily::Name("barlow_bold".into())),
    );
    style.text_styles.insert(
        TextStyle::Body,
        FontId::new(15.0, FontFamily::Proportional),
    );
    style.text_styles.insert(
        TextStyle::Button,
        FontId::new(14.0, FontFamily::Name("barlow_semibold".into())),
    );
    style.text_styles.insert(
        TextStyle::Small,
        FontId::new(12.0, FontFamily::Proportional),
    );
    style.text_styles.insert(
        TextStyle::Monospace,
        FontId::new(13.0, FontFamily::Monospace),
    );

    ctx.set_style(style);
}

pub fn title_font() -> FontId {
    FontId::new(28.0, FontFamily::Name("barlow_bold".into()))
}

pub fn tab_font() -> FontId {
    FontId::new(14.0, FontFamily::Name("barlow_semibold".into()))
}

pub fn section_font() -> FontId {
    FontId::new(13.0, FontFamily::Name("barlow_bold".into()))
}

pub fn row_font() -> FontId {
    FontId::new(15.0, FontFamily::Name("barlow_medium".into()))
}

pub fn button_font() -> FontId {
    FontId::new(13.0, FontFamily::Name("barlow_bold".into()))
}
