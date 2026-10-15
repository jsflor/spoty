use crate::theme::{COLOR_ACCENT_GREEN, COLOR_SLIDER_BG, COLOR_TEXT_BASE};
use eframe::egui::{self, Color32, CornerRadius, FontId, Pos2, Rect, Sense, Stroke, Vec2};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SliderTooltip {
    Time,
    Percent,
}

impl SliderTooltip {
    pub fn format_value(&self, val: f64) -> String {
        match self {
            SliderTooltip::Time => {
                let mins = (val / 60.0).floor() as u32;
                let secs = (val % 60.0).floor() as u32;
                format!("{}:{:02}", mins, secs)
            }
            SliderTooltip::Percent => {
                format!("{}%", (val * 100.0).round() as u32)
            }
        }
    }
}

pub struct SpotifySlider<'a> {
    value: &'a mut f64,
    max: f64,
    desired_width: Option<f32>,
    tooltip: Option<SliderTooltip>,
}

impl<'a> SpotifySlider<'a> {
    pub fn new(value: &'a mut f64, max: f64) -> Self {
        Self {
            value,
            max: if max > 0.0 { max } else { 1.0 },
            desired_width: None,
            tooltip: None,
        }
    }

    pub fn width(mut self, width: f32) -> Self {
        self.desired_width = Some(width);
        self
    }

    pub fn tooltip_time(mut self) -> Self {
        self.tooltip = Some(SliderTooltip::Time);
        self
    }

    pub fn tooltip_percent(mut self) -> Self {
        self.tooltip = Some(SliderTooltip::Percent);
        self
    }

    pub fn show(self, ui: &mut egui::Ui) -> egui::Response {
        let width = self
            .desired_width
            .unwrap_or_else(|| ui.available_width().max(40.0));
        let height = 14.0; // Comfortable interaction height

        let (rect, mut response) =
            ui.allocate_exact_size(Vec2::new(width, height), Sense::click_and_drag());
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
                painter.circle_stroke(
                    thumb_pos,
                    thumb_radius,
                    Stroke::new(1.0_f32, Color32::from_black_alpha(40)),
                );

                // Floating Tooltip Badge
                if let (Some(tooltip_mode), Some(mouse_pos)) = (self.tooltip, response.hover_pos())
                {
                    let hover_norm = ((mouse_pos.x - rect.left()) / rect.width()).clamp(0.0, 1.0);
                    let hover_val = hover_norm as f64 * self.max;

                    let text = tooltip_mode.format_value(hover_val);

                    let tip_center_x = mouse_pos.x.clamp(rect.left() + 20.0, rect.right() - 20.0);
                    let tip_center = Pos2::new(tip_center_x, rect.top() - 14.0);
                    let font_id = FontId::monospace(10.5);

                    let galley = painter.layout_no_wrap(text, font_id, Color32::WHITE);
                    let badge_size = Vec2::new(galley.size().x + 12.0, galley.size().y + 6.0);
                    let badge_rect = Rect::from_center_size(tip_center, badge_size);

                    painter.rect_filled(
                        badge_rect,
                        CornerRadius::same(4),
                        Color32::from_rgb(0x28, 0x28, 0x28),
                    );
                    painter.rect_stroke(
                        badge_rect,
                        CornerRadius::same(4),
                        Stroke::new(1.0_f32, Color32::from_rgb(0x40, 0x40, 0x40)),
                        egui::StrokeKind::Outside,
                    );
                    painter.galley(
                        Pos2::new(
                            badge_rect.center().x - (galley.size().x / 2.0),
                            badge_rect.center().y - (galley.size().y / 2.0),
                        ),
                        galley,
                        Color32::WHITE,
                    );
                }
            }
        }

        if changed {
            response.mark_changed();
        }

        response
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slider_tooltip_formatting() {
        assert_eq!(SliderTooltip::Time.format_value(0.0), "0:00");
        assert_eq!(SliderTooltip::Time.format_value(75.0), "1:15");
        assert_eq!(SliderTooltip::Time.format_value(3599.0), "59:59");

        assert_eq!(SliderTooltip::Percent.format_value(0.0), "0%");
        assert_eq!(SliderTooltip::Percent.format_value(0.75), "75%");
        assert_eq!(SliderTooltip::Percent.format_value(1.0), "100%");
    }
}
