use eframe::egui::{self, Color32, CornerRadius, Pos2, Rect, Sense, Stroke, Vec2};
use crate::theme::{COLOR_ACCENT_GREEN, COLOR_SLIDER_BG, COLOR_TEXT_BASE};

pub struct SpotifySlider<'a> {
    value: &'a mut f64,
    max: f64,
    desired_width: Option<f32>,
}

impl<'a> SpotifySlider<'a> {
    pub fn new(value: &'a mut f64, max: f64) -> Self {
        Self {
            value,
            max: if max > 0.0 { max } else { 1.0 },
            desired_width: None,
        }
    }

    pub fn width(mut self, width: f32) -> Self {
        self.desired_width = Some(width);
        self
    }

    pub fn show(self, ui: &mut egui::Ui) -> egui::Response {
        let width = self.desired_width.unwrap_or_else(|| ui.available_width().max(40.0));
        let height = 14.0; // Comfortable interaction height

        let (rect, mut response) = ui.allocate_exact_size(Vec2::new(width, height), Sense::click_and_drag());
        let mut changed = false;

        let is_hovered = response.hovered() || response.dragged();

        if response.clicked() || response.dragged() {
            if let Some(mouse_pos) = response.interact_pointer_pos() {
                let norm = ((mouse_pos.x - rect.left()) / rect.width()).clamp(0.0, 1.0);
                let new_val = norm as f64 * self.max;
                if (*self.value - new_val).abs() > 0.0001 {
                    *self.value = new_val;
                    changed = true;
                }
            }
        }

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            let track_height = 4.0;
            let track_y = rect.center().y - (track_height / 2.0);
            let track_rect = Rect::from_min_size(
                Pos2::new(rect.left(), track_y),
                Vec2::new(rect.width(), track_height),
            );

            // Background inactive bar
            painter.rect_filled(track_rect, CornerRadius::same(2), COLOR_SLIDER_BG);

            // Progress bar
            let progress_norm = (*self.value / self.max).clamp(0.0, 1.0) as f32;
            let progress_width = (rect.width() * progress_norm).max(0.0);
            let progress_rect = Rect::from_min_size(
                Pos2::new(rect.left(), track_y),
                Vec2::new(progress_width, track_height),
            );

            let fill_color = if is_hovered {
                COLOR_ACCENT_GREEN
            } else {
                COLOR_TEXT_BASE
            };

            painter.rect_filled(progress_rect, CornerRadius::same(2), fill_color);

            // Circular scrub thumb on hover or drag
            if is_hovered {
                let thumb_x = rect.left() + progress_width;
                let thumb_pos = Pos2::new(thumb_x, rect.center().y);
                let thumb_radius = 5.5;
                painter.circle_filled(thumb_pos, thumb_radius, Color32::WHITE);
                painter.circle_stroke(thumb_pos, thumb_radius, Stroke::new(1.0_f32, Color32::from_black_alpha(40)));
            }
        }

        if changed {
            response.mark_changed();
        }

        response
    }
}
