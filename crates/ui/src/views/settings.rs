use eframe::egui;

pub struct SettingsView;

impl SettingsView {
    pub fn show(
        ui: &mut egui::Ui,
        current_provider_name: &str,
        local_dir_input: &mut String,
        on_switch_to_local_fs: &mut impl FnMut(String),
        on_switch_to_demo: &mut impl FnMut(),
    ) {
        ui.heading(egui::RichText::new("Settings").size(26.0).strong().color(egui::Color32::WHITE));
        ui.add_space(16.0);

        ui.group(|ui| {
            ui.set_width(ui.available_width());
            ui.label(egui::RichText::new("Storage Provider").size(18.0).strong().color(egui::Color32::WHITE));
            ui.add_space(8.0);
            ui.label(format!("Active Source: {}", current_provider_name));
            ui.add_space(12.0);

            ui.horizontal(|ui| {
                if ui.button("Use Demo Catalog").clicked() {
                    on_switch_to_demo();
                }
            });

            ui.add_space(16.0);
            ui.separator();
            ui.add_space(12.0);

            ui.label(egui::RichText::new("Local Music Directory (OpenDAL)").strong().color(egui::Color32::WHITE));
            ui.label(egui::RichText::new("Scan a local folder on your Mac, PC, or Raspberry Pi").small().color(egui::Color32::GRAY));
            ui.add_space(6.0);

            ui.horizontal(|ui| {
                ui.add(
                    egui::TextEdit::singleline(local_dir_input)
                        .hint_text("/path/to/your/music")
                        .desired_width(360.0),
                );

                if ui.button("Scan Directory").clicked() {
                    let path = local_dir_input.trim().to_string();
                    if !path.is_empty() {
                        on_switch_to_local_fs(path);
                    }
                }
            });
        });

        ui.add_space(20.0);

        ui.group(|ui| {
            ui.set_width(ui.available_width());
            ui.label(egui::RichText::new("About Spoty").size(16.0).strong().color(egui::Color32::WHITE));
            ui.add_space(6.0);
            ui.label("Open-source, self-hosted music player built with Rust and egui.");
            ui.label("Designed for total ownership over your personal music library across S3, R2, Dropbox, and Local Storage.");
        });
    }
}
