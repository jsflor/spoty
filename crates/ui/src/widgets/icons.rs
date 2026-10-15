use crate::theme::{
    COLOR_ACCENT_GREEN, COLOR_ACCENT_GREEN_HOVER, COLOR_TEXT_BASE, COLOR_TEXT_SUBDUED,
};
use eframe::egui::{
    self, Color32, CornerRadius, FontId, Pos2, Rect, Response, Sense, Stroke, Vec2,
};
use spoty_core::RepeatMode;

/// Renders a circular Play/Pause button with Spotify-style hover scale
pub fn play_circle_button(
    ui: &mut egui::Ui,
    diameter: f32,
    is_playing: bool,
    bg_color: Option<Color32>,
) -> Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(diameter), Sense::click());

    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        let center = rect.center();
        let radius = diameter / 2.0;

        let is_hovered = response.hovered();
        let draw_radius = if is_hovered { radius + 1.0 } else { radius };

        let bg = bg_color.unwrap_or(Color32::WHITE);
        let actual_bg = if is_hovered {
            if bg == COLOR_ACCENT_GREEN {
                COLOR_ACCENT_GREEN_HOVER
            } else {
                Color32::from_rgb(0xe0, 0xe0, 0xe0)
            }
        } else {
            bg
        };

        painter.circle_filled(center, draw_radius, actual_bg);

        let icon_color = Color32::BLACK;
        let icon_scale = diameter * 0.32;

        if is_playing {
            // Two vertical pause bars
            let bar_w = icon_scale * 0.28;
            let bar_h = icon_scale * 0.95;
            let bar_spacing = icon_scale * 0.22;

            let bar1 = Rect::from_center_size(
                Pos2::new(center.x - bar_spacing, center.y),
                Vec2::new(bar_w, bar_h),
            );
            let bar2 = Rect::from_center_size(
                Pos2::new(center.x + bar_spacing, center.y),
                Vec2::new(bar_w, bar_h),
            );

            painter.rect_filled(bar1, CornerRadius::same(1), icon_color);
            painter.rect_filled(bar2, CornerRadius::same(1), icon_color);
        } else {
            // Play triangle (centered with slight optical offset to the right)
            let tri_h = icon_scale * 0.9;
            let tri_w = icon_scale * 0.75;
            let p1 = Pos2::new(center.x - tri_w * 0.38, center.y - tri_h / 2.0);
            let p2 = Pos2::new(center.x + tri_w * 0.62, center.y);
            let p3 = Pos2::new(center.x - tri_w * 0.38, center.y + tri_h / 2.0);

            painter.add(egui::Shape::convex_polygon(
                vec![p1, p2, p3],
                icon_color,
                Stroke::NONE,
            ));
        }
    }

    response
}

/// Renders a Spotify Shuffle button with an active green dot underneath
pub fn shuffle_button(ui: &mut egui::Ui, is_shuffle: bool) -> Response {
    let size = Vec2::splat(32.0);
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());

    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        let center = rect.center();

        let fg = if is_shuffle {
            COLOR_ACCENT_GREEN
        } else if response.hovered() {
            COLOR_TEXT_BASE
        } else {
            COLOR_TEXT_SUBDUED
        };

        draw_shuffle_icon(painter, Pos2::new(center.x, center.y - 1.0), fg);

        if is_shuffle {
            let dot_center = Pos2::new(center.x, rect.bottom() - 4.0);
            painter.circle_filled(dot_center, 2.0, COLOR_ACCENT_GREEN);
        }
    }

    response
}

/// Renders a vector Spotify shuffle icon (two crossed arrows)
pub fn draw_shuffle_icon(painter: &egui::Painter, center: Pos2, fg: Color32) {
    let stroke = Stroke::new(1.6_f32, fg);
    let y_top = center.y - 4.0;
    let y_bot = center.y + 4.0;
    let x_l = center.x - 7.0;
    let x_m1 = center.x - 2.0;
    let x_m2 = center.x + 2.0;
    let x_r = center.x + 5.0;

    // Arrow 1: top-left to bottom-right
    painter.line_segment([Pos2::new(x_l, y_top), Pos2::new(x_m1, y_top)], stroke);
    painter.line_segment([Pos2::new(x_m1, y_top), Pos2::new(x_m2, y_bot)], stroke);
    painter.line_segment([Pos2::new(x_m2, y_bot), Pos2::new(x_r, y_bot)], stroke);

    // Arrowhead 1 (bottom right)
    let ah1_tip = Pos2::new(center.x + 7.5, y_bot);
    let ah1_top = Pos2::new(center.x + 4.2, y_bot - 2.8);
    let ah1_bot = Pos2::new(center.x + 4.2, y_bot + 2.8);
    painter.add(egui::Shape::convex_polygon(
        vec![ah1_tip, ah1_top, ah1_bot],
        fg,
        Stroke::NONE,
    ));

    // Arrow 2: bottom-left to top-right (with gap at crossover)
    painter.line_segment([Pos2::new(x_l, y_bot), Pos2::new(x_m1, y_bot)], stroke);
    painter.line_segment(
        [
            Pos2::new(x_m1, y_bot),
            Pos2::new(center.x - 0.7, center.y + 1.4),
        ],
        stroke,
    );
    painter.line_segment(
        [
            Pos2::new(center.x + 0.7, center.y - 1.4),
            Pos2::new(x_m2, y_top),
        ],
        stroke,
    );
    painter.line_segment([Pos2::new(x_m2, y_top), Pos2::new(x_r, y_top)], stroke);

    // Arrowhead 2 (top right)
    let ah2_tip = Pos2::new(center.x + 7.5, y_top);
    let ah2_top = Pos2::new(center.x + 4.2, y_top - 2.8);
    let ah2_bot = Pos2::new(center.x + 4.2, y_top + 2.8);
    painter.add(egui::Shape::convex_polygon(
        vec![ah2_tip, ah2_top, ah2_bot],
        fg,
        Stroke::NONE,
    ));
}

/// Renders a crisp vector Home house icon
pub fn draw_home_icon(painter: &egui::Painter, center: Pos2, size: f32, color: Color32) {
    let half_w = size * 0.5;
    let half_h = size * 0.5;

    // Roof triangle
    let p_top = Pos2::new(center.x, center.y - half_h);
    let p_left = Pos2::new(center.x - half_w * 0.95, center.y - half_h * 0.05);
    let p_right = Pos2::new(center.x + half_w * 0.95, center.y - half_h * 0.05);
    painter.add(egui::Shape::convex_polygon(
        vec![p_top, p_left, p_right],
        color,
        Stroke::NONE,
    ));

    // House body rectangle
    let body_w = size * 0.72;
    let body_top = center.y - half_h * 0.08;
    let body_bottom = center.y + half_h * 0.85;
    let body_rect = Rect::from_min_max(
        Pos2::new(center.x - body_w / 2.0, body_top),
        Pos2::new(center.x + body_w / 2.0, body_bottom),
    );
    painter.rect_filled(body_rect, CornerRadius::same(1), color);
}

/// Renders a crisp vector Plus icon
pub fn draw_plus_icon(
    painter: &egui::Painter,
    center: Pos2,
    size: f32,
    stroke_width: f32,
    color: Color32,
) {
    let r = CornerRadius::same((stroke_width * 0.5).round() as u8);
    let h_bar = Rect::from_center_size(center, Vec2::new(size, stroke_width));
    let v_bar = Rect::from_center_size(center, Vec2::new(stroke_width, size));
    painter.rect_filled(h_bar, r, color);
    painter.rect_filled(v_bar, r, color);
}

/// Renders a crisp vector Close / Cross ('X') icon
pub fn draw_close_icon(
    painter: &egui::Painter,
    center: Pos2,
    size: f32,
    stroke_width: f32,
    color: Color32,
) {
    let d = size * 0.5;
    let stroke = Stroke::new(stroke_width, color);
    painter.line_segment(
        [
            Pos2::new(center.x - d, center.y - d),
            Pos2::new(center.x + d, center.y + d),
        ],
        stroke,
    );
    painter.line_segment(
        [
            Pos2::new(center.x - d, center.y + d),
            Pos2::new(center.x + d, center.y - d),
        ],
        stroke,
    );
}

/// Renders a crisp vector Sort Ascending / Descending arrow
pub fn draw_sort_arrow(
    painter: &egui::Painter,
    center: Pos2,
    ascending: bool,
    size: f32,
    color: Color32,
) {
    let half = size * 0.5;
    let (p1, p2, p3) = if ascending {
        (
            Pos2::new(center.x, center.y - half),
            Pos2::new(center.x - half * 0.85, center.y + half * 0.7),
            Pos2::new(center.x + half * 0.85, center.y + half * 0.7),
        )
    } else {
        (
            Pos2::new(center.x - half * 0.85, center.y - half * 0.7),
            Pos2::new(center.x + half * 0.85, center.y - half * 0.7),
            Pos2::new(center.x, center.y + half),
        )
    };
    painter.add(egui::Shape::convex_polygon(
        vec![p1, p2, p3],
        color,
        Stroke::NONE,
    ));
}

/// Renders a Spotify Repeat button with All / One active indicators
pub fn repeat_button(ui: &mut egui::Ui, repeat_mode: RepeatMode) -> Response {
    let size = Vec2::splat(32.0);
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());

    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        let center = rect.center();

        let is_active = repeat_mode != RepeatMode::Off;
        let fg = if is_active {
            COLOR_ACCENT_GREEN
        } else if response.hovered() {
            COLOR_TEXT_BASE
        } else {
            COLOR_TEXT_SUBDUED
        };

        painter.text(
            Pos2::new(center.x, center.y - 2.0),
            egui::Align2::CENTER_CENTER,
            "↻",
            FontId::proportional(19.0),
            fg,
        );

        if repeat_mode == RepeatMode::All {
            let dot_center = Pos2::new(center.x, rect.bottom() - 4.0);
            painter.circle_filled(dot_center, 2.0, COLOR_ACCENT_GREEN);
        } else if repeat_mode == RepeatMode::One {
            painter.text(
                Pos2::new(center.x + 8.0, center.y - 7.0),
                egui::Align2::CENTER_CENTER,
                "1",
                FontId::proportional(9.0),
                COLOR_ACCENT_GREEN,
            );
        }
    }

    response
}

/// Renders Previous track button (`|◀`)
pub fn prev_button(ui: &mut egui::Ui) -> Response {
    let size = Vec2::splat(32.0);
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());

    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        let center = rect.center();

        let fg = if response.hovered() {
            COLOR_TEXT_BASE
        } else {
            COLOR_TEXT_SUBDUED
        };

        // Draw bar on left + triangle pointing left
        let bar_rect =
            Rect::from_center_size(Pos2::new(center.x - 6.0, center.y), Vec2::new(2.5, 12.0));
        painter.rect_filled(bar_rect, CornerRadius::same(1), fg);

        let p1 = Pos2::new(center.x + 5.0, center.y - 6.0);
        let p2 = Pos2::new(center.x - 4.0, center.y);
        let p3 = Pos2::new(center.x + 5.0, center.y + 6.0);
        painter.add(egui::Shape::convex_polygon(
            vec![p1, p2, p3],
            fg,
            Stroke::NONE,
        ));
    }

    response
}

/// Renders Next track button (`▶|`)
pub fn next_button(ui: &mut egui::Ui) -> Response {
    let size = Vec2::splat(32.0);
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());

    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        let center = rect.center();

        let fg = if response.hovered() {
            COLOR_TEXT_BASE
        } else {
            COLOR_TEXT_SUBDUED
        };

        // Draw triangle pointing right + bar on right
        let p1 = Pos2::new(center.x - 5.0, center.y - 6.0);
        let p2 = Pos2::new(center.x + 4.0, center.y);
        let p3 = Pos2::new(center.x - 5.0, center.y + 6.0);
        painter.add(egui::Shape::convex_polygon(
            vec![p1, p2, p3],
            fg,
            Stroke::NONE,
        ));

        let bar_rect =
            Rect::from_center_size(Pos2::new(center.x + 6.0, center.y), Vec2::new(2.5, 12.0));
        painter.rect_filled(bar_rect, CornerRadius::same(1), fg);
    }

    response
}

/// Renders Heart / Favorite button
pub fn heart_button(ui: &mut egui::Ui, is_fav: bool) -> Response {
    let size = Vec2::splat(28.0);
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());

    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        let center = rect.center();

        let (icon_text, color) = if is_fav {
            ("♥", COLOR_ACCENT_GREEN)
        } else if response.hovered() {
            ("♡", COLOR_TEXT_BASE)
        } else {
            ("♡", COLOR_TEXT_SUBDUED)
        };

        painter.text(
            center,
            egui::Align2::CENTER_CENTER,
            icon_text,
            FontId::proportional(18.0),
            color,
        );
    }

    response
}

/// Renders Volume / Speaker icon
pub fn speaker_button(ui: &mut egui::Ui, is_muted: bool, volume: f32) -> Response {
    let size = Vec2::splat(28.0);
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());

    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        let center = rect.center();

        let fg = if response.hovered() {
            COLOR_TEXT_BASE
        } else {
            COLOR_TEXT_SUBDUED
        };

        let icon = if is_muted || volume == 0.0 {
            "🔇"
        } else if volume < 0.35 {
            "🔈"
        } else if volume < 0.70 {
            "🔉"
        } else {
            "🔊"
        };

        painter.text(
            center,
            egui::Align2::CENTER_CENTER,
            icon,
            FontId::proportional(15.0),
            fg,
        );
    }

    response
}

/// Renders Queue button (`≡`) with active indicator
pub fn queue_button(ui: &mut egui::Ui, is_open: bool) -> Response {
    let size = Vec2::splat(32.0);
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());

    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        let center = rect.center();

        let fg = if is_open {
            COLOR_ACCENT_GREEN
        } else if response.hovered() {
            COLOR_TEXT_BASE
        } else {
            COLOR_TEXT_SUBDUED
        };

        // Draw 3 horizontal bars (queue icon)
        let bar_width = 15.0;
        let bar_height = 2.0;
        let y_offsets = [-4.5, 0.0, 4.5];

        for y_off in y_offsets {
            let bar_rect = Rect::from_center_size(
                Pos2::new(center.x, center.y + y_off),
                Vec2::new(bar_width, bar_height),
            );
            painter.rect_filled(bar_rect, CornerRadius::same(1), fg);
        }

        // Active dot underneath if open
        if is_open {
            let dot_center = Pos2::new(center.x, rect.bottom() - 2.0);
            painter.circle_filled(dot_center, 1.5, COLOR_ACCENT_GREEN);
        }
    }

    response
}
