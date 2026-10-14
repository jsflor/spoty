use egui::{Color32, CornerRadius, Stroke, Visuals};

// Spotify Encore Surface Tokens
pub const COLOR_CANVAS: Color32 = Color32::from_rgb(0x00, 0x00, 0x00);
pub const COLOR_BG_BASE: Color32 = Color32::from_rgb(0x12, 0x12, 0x12);
pub const COLOR_BG_CARD: Color32 = Color32::from_rgb(0x18, 0x18, 0x18);
pub const COLOR_BG_ELEVATED: Color32 = Color32::from_rgb(0x1f, 0x1f, 0x1f);
pub const COLOR_BG_HIGHLIGHT: Color32 = Color32::from_rgb(0x2a, 0x2a, 0x2a);
pub const COLOR_BG_PRESS: Color32 = Color32::from_rgb(0x19, 0x19, 0x19);
#[allow(dead_code)]
pub const COLOR_BG_TINTED: Color32 = Color32::from_rgba_premultiplied(0xff, 0xff, 0xff, 0x1a);

// Spotify Encore Typography & Accent Tokens
pub const COLOR_TEXT_BASE: Color32 = Color32::from_rgb(0xff, 0xff, 0xff);
pub const COLOR_TEXT_SUBDUED: Color32 = Color32::from_rgb(0xb3, 0xb3, 0xb3);
#[allow(dead_code)]
pub const COLOR_TEXT_MUTED: Color32 = Color32::from_rgb(0x7c, 0x7c, 0x7c);

pub const COLOR_ACCENT_GREEN: Color32 = Color32::from_rgb(0x1e, 0xd7, 0x60);
pub const COLOR_ACCENT_GREEN_HOVER: Color32 = Color32::from_rgb(0x3b, 0xe4, 0x77);
pub const COLOR_SLIDER_BG: Color32 = Color32::from_rgb(0x4d, 0x4d, 0x4d);
pub const COLOR_DIVIDER: Color32 = Color32::from_rgba_premultiplied(0xff, 0xff, 0xff, 0x15);

pub fn setup_spotify_theme(ctx: &egui::Context) {
    let mut visuals = Visuals::dark();

    visuals.panel_fill = COLOR_CANVAS;
    visuals.window_fill = COLOR_BG_ELEVATED;
    visuals.faint_bg_color = COLOR_BG_CARD;
    visuals.extreme_bg_color = COLOR_BG_BASE;

    // Default widgets styling with 8px Spotify card radius
    visuals.widgets.noninteractive.bg_fill = COLOR_BG_BASE;
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0_f32, COLOR_TEXT_BASE);

    visuals.widgets.inactive.bg_fill = COLOR_BG_CARD;
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0_f32, COLOR_TEXT_SUBDUED);
    visuals.widgets.inactive.corner_radius = CornerRadius::same(8);

    visuals.widgets.hovered.bg_fill = COLOR_BG_HIGHLIGHT;
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0_f32, COLOR_TEXT_BASE);
    visuals.widgets.hovered.corner_radius = CornerRadius::same(8);

    visuals.widgets.active.bg_fill = COLOR_BG_PRESS;
    visuals.widgets.active.fg_stroke = Stroke::new(1.0_f32, COLOR_ACCENT_GREEN);
    visuals.widgets.active.corner_radius = CornerRadius::same(8);

    visuals.selection.bg_fill = COLOR_ACCENT_GREEN;
    visuals.selection.stroke = Stroke::new(1.0_f32, COLOR_CANVAS);

    ctx.set_visuals(visuals);
}
