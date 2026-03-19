use crate::app::MashApp;
use crate::mhl::types::DiffResult;

const ACCENT: egui::Color32 = egui::Color32::from_rgb(80, 160, 255);
const TEXT_DIM: egui::Color32 = egui::Color32::from_rgb(160, 160, 170);
const GREEN: egui::Color32 = egui::Color32::from_rgb(80, 200, 120);
const RED: egui::Color32 = egui::Color32::from_rgb(240, 80, 80);
const ORANGE: egui::Color32 = egui::Color32::from_rgb(240, 180, 50);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffFilter {
    All,
    Added,
    Removed,
    Modified,
    Unchanged,
}

impl MashApp {
    pub fn compare_panel(&mut self, ui: &mut egui::Ui) {
        // File selection
        egui::Frame::group(ui.style())
            .inner_margin(egui::Margin::same(12))
            .show(ui, |ui| {
                ui.label(
                    egui::RichText::new("Compare Manifests")
                        .strong()
                        .size(14.0)
                        .color(ACCENT),
                );
                ui.add_space(4.0);
                ui.label(
                    egui::RichText::new("Load two CSV, TSV, or MHL files to compare")
                        .size(12.0)
                        .color(TEXT_DIM),
                );
                ui.add_space(6.0);

                // File A
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("A:").strong().size(13.0));
                    ui.add(
                        egui::TextEdit::singleline(&mut self.compare_file_a)
                            .desired_width(ui.available_width() - 80.0)
                            .hint_text("First manifest..."),
                    );
                    if ui.button("Browse").clicked() {
                        if let Some(path) = manifest_file_dialog() {
                            self.compare_file_a = path;
                        }
                    }
                });

                // File B
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("B:").strong().size(13.0));
                    ui.add(
                        egui::TextEdit::singleline(&mut self.compare_file_b)
                            .desired_width(ui.available_width() - 80.0)
                            .hint_text("Second manifest..."),
                    );
                    if ui.button("Browse").clicked() {
                        if let Some(path) = manifest_file_dialog() {
                            self.compare_file_b = path;
                        }
                    }
                });
            });

        ui.add_space(6.0);

        // Error
        if let Some(ref err) = self.compare_error {
            egui::Frame::group(ui.style())
                .inner_margin(egui::Margin::same(12))
                .show(ui, |ui| {
                    ui.label(egui::RichText::new(err).color(RED));
                });
            ui.add_space(6.0);
        }

        // Results
        if let Some(ref result) = self.compare_result.clone() {
            self.compare_results_panel(ui, &result);
        }
    }

    fn compare_results_panel(&mut self, ui: &mut egui::Ui, result: &DiffResult) {
        // Summary bar
        egui::Frame::group(ui.style())
            .inner_margin(egui::Margin::same(12))
            .show(ui, |ui| {
                ui.label(
                    egui::RichText::new("Results")
                        .strong()
                        .size(14.0)
                        .color(ACCENT),
                );
                ui.add_space(4.0);

                ui.horizontal_wrapped(|ui| {
                    let filters = [
                        (
                            DiffFilter::All,
                            format!(
                                "All ({})",
                                result.added.len()
                                    + result.removed.len()
                                    + result.modified.len()
                                    + result.unchanged.len()
                            ),
                            egui::Color32::WHITE,
                        ),
                        (
                            DiffFilter::Added,
                            format!("Added ({})", result.added.len()),
                            GREEN,
                        ),
                        (
                            DiffFilter::Removed,
                            format!("Removed ({})", result.removed.len()),
                            RED,
                        ),
                        (
                            DiffFilter::Modified,
                            format!("Modified ({})", result.modified.len()),
                            ORANGE,
                        ),
                        (
                            DiffFilter::Unchanged,
                            format!("Unchanged ({})", result.unchanged.len()),
                            TEXT_DIM,
                        ),
                    ];

                    for (filter, label, color) in filters {
                        let is_selected = self.compare_filter == filter;
                        let text = if is_selected {
                            egui::RichText::new(&label).strong().color(color)
                        } else {
                            egui::RichText::new(&label).color(color)
                        };

                        let btn = egui::Button::new(text);
                        if ui.add(btn).clicked() {
                            self.compare_filter = filter;
                        }
                    }
                });
            });

        ui.add_space(6.0);

        // File list
        egui::Frame::group(ui.style())
            .inner_margin(egui::Margin::same(12))
            .show(ui, |ui| {
                egui::ScrollArea::vertical()
                    .max_height(400.0)
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        let show_added = matches!(
                            self.compare_filter,
                            DiffFilter::All | DiffFilter::Added
                        );
                        let show_removed = matches!(
                            self.compare_filter,
                            DiffFilter::All | DiffFilter::Removed
                        );
                        let show_modified = matches!(
                            self.compare_filter,
                            DiffFilter::All | DiffFilter::Modified
                        );
                        let show_unchanged = matches!(
                            self.compare_filter,
                            DiffFilter::All | DiffFilter::Unchanged
                        );

                        let mut any_rows = false;

                        if show_added {
                            for entry in &result.added {
                                any_rows = true;
                                ui.horizontal(|ui| {
                                    ui.label(
                                        egui::RichText::new("+")
                                            .strong()
                                            .monospace()
                                            .color(GREEN),
                                    );
                                    ui.label(
                                        egui::RichText::new(&entry.path).color(GREEN),
                                    );
                                    if let Some((algo, val)) = entry.hashes.first() {
                                        ui.label(
                                            egui::RichText::new(format!(
                                                "{}:{}",
                                                algo.xml_tag(),
                                                truncate_hash(val)
                                            ))
                                            .monospace()
                                            .size(11.0)
                                            .color(TEXT_DIM),
                                        );
                                    }
                                });
                            }
                        }

                        if show_removed {
                            for entry in &result.removed {
                                any_rows = true;
                                ui.horizontal(|ui| {
                                    ui.label(
                                        egui::RichText::new("-")
                                            .strong()
                                            .monospace()
                                            .color(RED),
                                    );
                                    ui.label(
                                        egui::RichText::new(&entry.path).color(RED),
                                    );
                                    if let Some((algo, val)) = entry.hashes.first() {
                                        ui.label(
                                            egui::RichText::new(format!(
                                                "{}:{}",
                                                algo.xml_tag(),
                                                truncate_hash(val)
                                            ))
                                            .monospace()
                                            .size(11.0)
                                            .color(TEXT_DIM),
                                        );
                                    }
                                });
                            }
                        }

                        if show_modified {
                            for entry in &result.modified {
                                any_rows = true;
                                ui.horizontal(|ui| {
                                    ui.label(
                                        egui::RichText::new("~")
                                            .strong()
                                            .monospace()
                                            .color(ORANGE),
                                    );
                                    ui.label(
                                        egui::RichText::new(&entry.path).color(ORANGE),
                                    );
                                });
                                // Show differing hashes indented
                                for (algo, val_a) in &entry.hashes_a {
                                    let val_b = entry
                                        .hashes_b
                                        .iter()
                                        .find(|(a, _)| a == algo)
                                        .map(|(_, v)| v.as_str());
                                    if let Some(vb) = val_b {
                                        if !val_a.eq_ignore_ascii_case(vb) {
                                            ui.horizontal(|ui| {
                                                ui.add_space(20.0);
                                                ui.label(
                                                    egui::RichText::new(format!(
                                                        "{} A: {}",
                                                        algo.xml_tag(),
                                                        val_a
                                                    ))
                                                    .monospace()
                                                    .size(11.0)
                                                    .color(RED),
                                                );
                                            });
                                            ui.horizontal(|ui| {
                                                ui.add_space(20.0);
                                                ui.label(
                                                    egui::RichText::new(format!(
                                                        "{} B: {}",
                                                        algo.xml_tag(),
                                                        vb
                                                    ))
                                                    .monospace()
                                                    .size(11.0)
                                                    .color(GREEN),
                                                );
                                            });
                                        }
                                    }
                                }
                            }
                        }

                        if show_unchanged {
                            for entry in &result.unchanged {
                                any_rows = true;
                                ui.horizontal(|ui| {
                                    ui.label(
                                        egui::RichText::new("=")
                                            .strong()
                                            .monospace()
                                            .color(TEXT_DIM),
                                    );
                                    ui.label(
                                        egui::RichText::new(&entry.path).color(TEXT_DIM),
                                    );
                                });
                            }
                        }

                        if !any_rows {
                            ui.label(
                                egui::RichText::new("No entries for this filter")
                                    .color(TEXT_DIM),
                            );
                        }
                    });
            });
    }
}

fn manifest_file_dialog() -> Option<String> {
    rfd::FileDialog::new()
        .add_filter("Manifests", &["csv", "tsv", "mhl"])
        .add_filter("All Files", &["*"])
        .pick_file()
        .map(|p| p.to_string_lossy().to_string())
}

fn truncate_hash(hash: &str) -> &str {
    if hash.len() > 16 {
        &hash[..16]
    } else {
        hash
    }
}
