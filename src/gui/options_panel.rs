use crate::app::MashApp;
use crate::mhl::types::{ChecksumColumn, HashAlgorithm};

const ACCENT: egui::Color32 = egui::Color32::from_rgb(80, 160, 255);
const TEXT_DIM: egui::Color32 = egui::Color32::from_rgb(160, 160, 170);

impl MashApp {
    /// Algorithm selection (radio buttons).
    pub fn algorithm_panel(&mut self, ui: &mut egui::Ui) {
        egui::Frame::group(ui.style())
            .inner_margin(egui::Margin::same(12))
            .show(ui, |ui| {
                ui.label(egui::RichText::new("Algorithm").strong().size(14.0).color(ACCENT));
                ui.add_space(4.0);

                ui.horizontal_wrapped(|ui| {
                    ui.radio_value(&mut self.selected_algorithm, HashAlgorithm::Xxh64, "XXH64");
                    ui.radio_value(&mut self.selected_algorithm, HashAlgorithm::Xxh128, "XXH128");
                    ui.radio_value(&mut self.selected_algorithm, HashAlgorithm::Xxh3, "XXH3");
                    ui.radio_value(&mut self.selected_algorithm, HashAlgorithm::Md5, "MD5");
                    ui.radio_value(&mut self.selected_algorithm, HashAlgorithm::Sha1, "SHA-1");
                });
            });
    }

    /// Output type toggles (MHL, Checksum file, Per-file).
    pub fn output_toggles_panel(&mut self, ui: &mut egui::Ui) {
        egui::Frame::group(ui.style())
            .inner_margin(egui::Margin::same(12))
            .show(ui, |ui| {
                ui.label(egui::RichText::new("Output").strong().size(14.0).color(ACCENT));
                ui.add_space(4.0);

                ui.horizontal(|ui| {
                    ui.checkbox(&mut self.generate_mhl, "MHL manifest");
                    ui.add_space(16.0);
                    ui.checkbox(&mut self.generate_checksum, "Checksum file");
                    ui.add_space(16.0);
                    ui.checkbox(&mut self.generate_per_file, "Per-file sidecar");
                });
            });
    }

    /// Checksum file and per-file sidecar settings (columns, format, etc.).
    pub fn checksum_settings_panel(&mut self, ui: &mut egui::Ui) {
        egui::Frame::group(ui.style())
            .inner_margin(egui::Margin::same(12))
            .show(ui, |ui| {
                // --- Checksum file settings ---
                if self.generate_checksum {
                    ui.label(
                        egui::RichText::new("Checksum File Settings")
                            .strong()
                            .size(14.0)
                            .color(ACCENT),
                    );
                    ui.add_space(4.0);

                    ui.horizontal(|ui| {
                        ui.label("Format:");
                        ui.radio_value(&mut self.checksum_separator, ',', "CSV");
                        ui.radio_value(&mut self.checksum_separator, '\t', "TSV");
                    });

                    ui.add_space(6.0);
                    ui.label("Columns:");
                    ui.add_space(2.0);

                    column_editor(ui, &mut self.selected_columns, "checksum_cols");
                }

                // Separator between sections if both visible
                if self.generate_checksum && self.generate_per_file {
                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(4.0);
                }

                // --- Per-file sidecar settings ---
                if self.generate_per_file {
                    ui.label(
                        egui::RichText::new("Per-file Sidecar Settings")
                            .strong()
                            .size(14.0)
                            .color(ACCENT),
                    );
                    ui.add_space(4.0);

                    ui.horizontal(|ui| {
                        ui.label("Format:");
                        ui.radio_value(&mut self.per_file_separator, ',', "CSV");
                        ui.radio_value(&mut self.per_file_separator, '\t', "TSV");
                        ui.add_space(16.0);
                        ui.checkbox(&mut self.per_file_header, "Include header row");
                    });

                    ui.horizontal(|ui| {
                        ui.label("Extension:");
                        ui.label(
                            egui::RichText::new("filename.mov.")
                                .monospace()
                                .size(12.0)
                                .color(TEXT_DIM),
                        );
                        ui.add(
                            egui::TextEdit::singleline(&mut self.per_file_extension)
                                .desired_width(60.0),
                        );
                    });

                    ui.add_space(6.0);
                    ui.label("Columns:");
                    ui.add_space(2.0);

                    column_editor(ui, &mut self.per_file_columns, "perfile_cols");
                }
            });
    }

}

/// Reusable column editor with checkboxes and up/down reorder buttons.
fn column_editor(ui: &mut egui::Ui, selected: &mut Vec<ChecksumColumn>, id_salt: &str) {
    let mut move_action: Option<(usize, isize)> = None;
    let mut toggle_action: Option<ChecksumColumn> = None;

    let all_cols = ChecksumColumn::all();

    let mut ordered: Vec<ChecksumColumn> = selected.clone();
    for col in all_cols {
        if !ordered.contains(col) {
            ordered.push(*col);
        }
    }

    for (i, col) in ordered.iter().enumerate() {
        let is_selected = selected.contains(col);
        let selected_idx = selected.iter().position(|c| c == col);

        ui.horizontal(|ui| {
            ui.push_id(format!("{}_{}", id_salt, i), |ui| {
                let mut checked = is_selected;
                if ui.checkbox(&mut checked, "").changed() {
                    toggle_action = Some(*col);
                }

                if is_selected {
                    let idx = selected_idx.unwrap();
                    let can_up = idx > 0;
                    let can_down = idx + 1 < selected.len();

                    let up_btn = egui::Button::new(
                        egui::RichText::new("^").monospace().size(11.0),
                    )
                    .min_size(egui::vec2(22.0, 20.0));
                    let dn_btn = egui::Button::new(
                        egui::RichText::new("v").monospace().size(11.0),
                    )
                    .min_size(egui::vec2(22.0, 20.0));

                    if ui.add_enabled(can_up, up_btn).clicked() {
                        move_action = Some((idx, -1));
                    }
                    if ui.add_enabled(can_down, dn_btn).clicked() {
                        move_action = Some((idx, 1));
                    }
                } else {
                    ui.add_enabled(
                        false,
                        egui::Button::new(egui::RichText::new("^").monospace().size(11.0))
                            .min_size(egui::vec2(22.0, 20.0)),
                    );
                    ui.add_enabled(
                        false,
                        egui::Button::new(egui::RichText::new("v").monospace().size(11.0))
                            .min_size(egui::vec2(22.0, 20.0)),
                    );
                }

                let text = if is_selected {
                    egui::RichText::new(col.label())
                } else {
                    egui::RichText::new(col.label()).color(TEXT_DIM)
                };
                ui.label(text);
            });
        });
    }

    if let Some(col) = toggle_action {
        if selected.contains(&col) {
            selected.retain(|c| *c != col);
        } else {
            selected.push(col);
        }
    }

    if let Some((idx, dir)) = move_action {
        let new_idx = (idx as isize + dir) as usize;
        if new_idx < selected.len() {
            selected.swap(idx, new_idx);
        }
    }
}
