use eframe::egui;
use spoty_storage::{S3Config, SubsonicConfig};
use crate::config::SpotyConfig;

pub struct SettingsView;

impl SettingsView {
    pub fn show(
        ui: &mut egui::Ui,
        current_provider_name: &str,
        config: &mut SpotyConfig,
        on_switch_to_demo: &mut impl FnMut(),
        on_switch_to_local_fs: &mut impl FnMut(String),
        on_switch_to_s3: &mut impl FnMut(S3Config),
        on_switch_to_subsonic: &mut impl FnMut(SubsonicConfig),
    ) {
        ui.heading(egui::RichText::new("Settings & Storage Backends").size(26.0).strong().color(egui::Color32::WHITE));
        ui.add_space(8.0);
        ui.label(egui::RichText::new("Configure your personal music sources across local drives, home servers, or cloud storage.").color(egui::Color32::GRAY));
        ui.add_space(16.0);

        // Active provider banner
        egui::Frame::new()
            .fill(egui::Color32::from_rgb(0x1a, 0x1a, 0x1a))
            .inner_margin(egui::Margin::same(12))
            .corner_radius(egui::CornerRadius::same(6))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Active Source:").strong().color(egui::Color32::WHITE));
                    ui.label(
                        egui::RichText::new(current_provider_name)
                            .strong()
                            .color(egui::Color32::from_rgb(0x1e, 0xd7, 0x60)),
                    );

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Reset to Demo Catalog").clicked() {
                            on_switch_to_demo();
                        }
                    });
                });
            });

        ui.add_space(20.0);

        // Section 1: Local Filesystem (Raspberry Pi / Local disk)
        ui.group(|ui| {
            ui.set_width(ui.available_width());
            ui.label(egui::RichText::new("1. Local Storage (OpenDAL)").size(16.0).strong().color(egui::Color32::WHITE));
            ui.label(egui::RichText::new("Index audio files stored on your local drive, external SSD, or a mounted home NAS").small().color(egui::Color32::GRAY));
            ui.add_space(8.0);

            ui.horizontal(|ui| {
                ui.add(
                    egui::TextEdit::singleline(&mut config.local_path)
                        .hint_text("/Users/username/Music or /media/music")
                        .desired_width(380.0),
                );

                if ui.button("Scan & Mount Folder").clicked() {
                    let path = config.local_path.trim().to_string();
                    if !path.is_empty() {
                        on_switch_to_local_fs(path);
                    }
                }
            });
        });

        ui.add_space(16.0);

        // Section 2: S3-Compatible Cloud Storage (Cloudflare R2, AWS S3, MinIO)
        ui.group(|ui| {
            ui.set_width(ui.available_width());
            ui.label(egui::RichText::new("2. S3 & Cloudflare R2 (OpenDAL)").size(16.0).strong().color(egui::Color32::WHITE));
            ui.label(egui::RichText::new("Stream high-res audio directly from an S3-compatible cloud bucket").small().color(egui::Color32::GRAY));
            ui.add_space(8.0);

            egui::Grid::new("s3_settings_grid")
                .spacing([12.0, 8.0])
                .min_col_width(120.0)
                .show(ui, |ui| {
                    ui.label("Bucket Name:");
                    ui.add(egui::TextEdit::singleline(&mut config.s3.bucket).hint_text("my-music-bucket").desired_width(280.0));
                    ui.end_row();

                    ui.label("Endpoint (R2/S3):");
                    let mut ep = config.s3.endpoint.clone().unwrap_or_default();
                    if ui.add(egui::TextEdit::singleline(&mut ep).hint_text("https://<account>.r2.cloudflarestorage.com").desired_width(280.0)).changed() {
                        config.s3.endpoint = if ep.is_empty() { None } else { Some(ep) };
                    }
                    ui.end_row();

                    ui.label("Access Key ID:");
                    let mut key = config.s3.access_key_id.clone().unwrap_or_default();
                    if ui.add(egui::TextEdit::singleline(&mut key).hint_text("Optional access key").desired_width(280.0)).changed() {
                        config.s3.access_key_id = if key.is_empty() { None } else { Some(key) };
                    }
                    ui.end_row();

                    ui.label("Secret Access Key:");
                    let mut secret = config.s3.secret_access_key.clone().unwrap_or_default();
                    if ui.add(egui::TextEdit::singleline(&mut secret).password(true).hint_text("Optional secret key").desired_width(280.0)).changed() {
                        config.s3.secret_access_key = if secret.is_empty() { None } else { Some(secret) };
                    }
                    ui.end_row();
                });

            ui.add_space(8.0);
            if ui.button("Connect & Mount S3 Bucket").clicked() {
                on_switch_to_s3(config.s3.clone());
            }
        });

        ui.add_space(16.0);

        // Section 3: Subsonic / Navidrome Home Server
        ui.group(|ui| {
            ui.set_width(ui.available_width());
            ui.label(egui::RichText::new("3. Navidrome / Subsonic Home Server").size(16.0).strong().color(egui::Color32::WHITE));
            ui.label(egui::RichText::new("Connect to your Raspberry Pi or home server running Navidrome, Jellyfin, or Airsonic").small().color(egui::Color32::GRAY));
            ui.add_space(8.0);

            egui::Grid::new("subsonic_settings_grid")
                .spacing([12.0, 8.0])
                .min_col_width(120.0)
                .show(ui, |ui| {
                    ui.label("Server URL:");
                    ui.add(egui::TextEdit::singleline(&mut config.subsonic.server_url).hint_text("http://raspberrypi:4533").desired_width(280.0));
                    ui.end_row();

                    ui.label("Username:");
                    ui.add(egui::TextEdit::singleline(&mut config.subsonic.username).hint_text("admin").desired_width(280.0));
                    ui.end_row();

                    ui.label("Password:");
                    ui.add(egui::TextEdit::singleline(&mut config.subsonic.password).password(true).desired_width(280.0));
                    ui.end_row();
                });

            ui.add_space(8.0);
            if ui.button("Test Connection & Sync Library").clicked() {
                on_switch_to_subsonic(config.subsonic.clone());
            }
        });
    }
}
