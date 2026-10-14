use egui::{Color32, CornerRadius, Stroke, Visuals};

pub const COLOR_BG_BASE: Color32 = Color32::from_rgb(0x12, 0x12, 0x12);
pub const COLOR_BG_CARD: Color32 = Color32::from_rgb(0x18, 0x18, 0x18);
pub const COLOR_BG_ELEVATED: Color32 = Color32::from_rgb(0x24, 0x24, 0x24);
pub const COLOR_BG_HIGHLIGHT: Color32 = Color32::from_rgb(0x2a, 0x2a, 0x2a);

pub const COLOR_TEXT_BASE: Color32 = Color32::from_rgb(0xff, 0xff, 0xff);
pub const COLOR_TEXT_SUBDUED: Color32 = Color32::from_rgb(0xa7, 0xa7, 0xa7);

pub const COLOR_ACCENT_GREEN: Color32 = Color32::from_rgb(0x1e, 0xd7, 0x60);
#[allow(dead_code)]
pub const COLOR_ACCENT_GREEN_HOVER: Color32 = Color32::from_rgb(0x3b, 0xe4, 0x77);

pub fn setup_spotify_theme(ctx: &egui::Context) {
    let mut visuals = Visuals::dark();

    visuals.panel_fill = COLOR_BG_BASE;
    visuals.window_fill = COLOR_BG_ELEVATED;
    visuals.faint_bg_color = COLOR_BG_CARD;
    visuals.extreme_bg_color = Color32::from_rgb(0x0a, 0x0a, 0x0a);

    // Widgets styling
    visuals.widgets.noninteractive.bg_fill = COLOR_BG_BASE;
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0_f32, COLOR_TEXT_BASE);

    visuals.widgets.inactive.bg_fill = COLOR_BG_CARD;
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0_f32, COLOR_TEXT_SUBDUED);
    visuals.widgets.inactive.corner_radius = CornerRadius::same(6);

    visuals.widgets.hovered.bg_fill = COLOR_BG_HIGHLIGHT;
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0_f32, COLOR_TEXT_BASE);
    visuals.widgets.hovered.corner_radius = CornerRadius::same(6);

    visuals.widgets.active.bg_fill = COLOR_BG_ELEVATED;
    visuals.widgets.active.fg_stroke = Stroke::new(1.0_f32, COLOR_ACCENT_GREEN);
    visuals.widgets.active.corner_radius = CornerRadius::same(6);

    visuals.selection.bg_fill = COLOR_ACCENT_GREEN;
    visuals.selection.stroke = Stroke::new(1.0_f32, COLOR_BG_BASE);

    ctx.set_visuals(visuals);
}
