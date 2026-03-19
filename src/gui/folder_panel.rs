use crate::app::MashApp;
use crate::util;

impl MashApp {
    pub fn folder_panel(&mut self, ui: &mut egui::Ui) {
        egui::Frame::group(ui.style())
            .inner_margin(egui::Margin::same(12))
            .show(ui, |ui| {
                ui.label(
                    egui::RichText::new("Target")
                        .strong()
                        .size(14.0)
                        .color(egui::Color32::from_rgb(80, 160, 255)),
                );
                ui.add_space(4.0);

                ui.horizontal(|ui| {
                    let path_edit = egui::TextEdit::singleline(&mut self.folder_path)
                        .desired_width(ui.available_width() - 170.0)
                        .hint_text("Select a folder or file to hash...");
                    let response = ui.add(path_edit);

                    if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        self.scan_folder();
                    }

                    if ui
                        .add(egui::Button::new("Folder").min_size(egui::vec2(60.0, 0.0)))
                        .clicked()
                    {
                        if let Some(path) = rfd::FileDialog::new().pick_folder() {
                            self.folder_path = path.to_string_lossy().to_string();
                            self.scan_folder();
                        }
                    }

                    if ui
                        .add(egui::Button::new("File").min_size(egui::vec2(50.0, 0.0)))
                        .clicked()
                    {
                        if let Some(path) = rfd::FileDialog::new().pick_file() {
                            self.folder_path = path.to_string_lossy().to_string();
                            self.scan_folder();
                        }
                    }
                });

                if self.folder_scanned {
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(format!(
                                "{} {}",
                                util::format_number(self.file_count),
                                if self.file_count == 1 { "file" } else { "files" }
                            ))
                            .color(egui::Color32::from_rgb(160, 160, 170)),
                        );
                        ui.label(
                            egui::RichText::new("|")
                                .color(egui::Color32::from_rgb(70, 70, 80)),
                        );
                        ui.label(
                            egui::RichText::new(util::format_bytes(self.total_size))
                                .color(egui::Color32::from_rgb(160, 160, 170)),
                        );
                    });
                }
            });
    }
}
