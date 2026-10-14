use eframe::egui;

pub struct VisualizerWidget<'a> {
    levels: &'a [f32; 16],
    is_playing: bool,
}

impl<'a> VisualizerWidget<'a> {
    pub fn new(levels: &'a [f32; 16], is_playing: bool) -> Self {
        Self { levels, is_playing }
    }

    pub fn show(self, ui: &mut egui::Ui) -> egui::Response {
        let (rect, response) = ui.allocate_exact_size(egui::vec2(80.0, 26.0), egui::Sense::hover());

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            let num_bars = 16;
            let spacing = 2.0;
            let total_spacing = spacing * (num_bars as f32 - 1.0);
            let bar_width = (rect.width() - total_spacing) / num_bars as f32;
            let max_height = rect.height() - 4.0;

            let accent_color = egui::Color32::from_rgb(0x1e, 0xd7, 0x60); // Spotify green
            let idle_color = egui::Color32::from_rgb(0x38, 0x38, 0x38);

            for (i, &level) in self.levels.iter().enumerate() {
                let x = rect.left() + i as f32 * (bar_width + spacing);
                let height = if self.is_playing {
                    (level * max_height).clamp(3.0, max_height)
                } else {
                    2.0
                };
                let y = rect.bottom() - height - 2.0;

                let bar_rect =
                    egui::Rect::from_min_size(egui::pos2(x, y), egui::vec2(bar_width, height));

                let color = if self.is_playing {
                    accent_color
                } else {
                    idle_color
                };
                painter.rect_filled(bar_rect, egui::CornerRadius::same(1), color);
            }
        }

        response
    }
}
