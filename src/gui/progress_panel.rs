use crate::app::{AppState, MashApp};
use crate::util;
use std::sync::atomic::Ordering;

const ACCENT: egui::Color32 = egui::Color32::from_rgb(80, 160, 255);
const TEXT_DIM: egui::Color32 = egui::Color32::from_rgb(160, 160, 170);
const SUCCESS: egui::Color32 = egui::Color32::from_rgb(80, 200, 120);
const ERROR_RED: egui::Color32 = egui::Color32::from_rgb(240, 80, 80);

impl MashApp {
    pub fn progress_panel(&mut self, ui: &mut egui::Ui) {
        match &self.state {
            AppState::Hashing { job, .. } => {
                let progress = &job.progress;
                let total_bytes = progress.total_bytes.load(Ordering::Relaxed);
                let processed_bytes = progress.processed_bytes.load(Ordering::Relaxed);
                let total_files = progress.total_files.load(Ordering::Relaxed);
                let processed_files = progress.processed_files.load(Ordering::Relaxed);
                let current_file = progress
                    .current_file
                    .lock()
                    .map(|s| s.clone())
                    .unwrap_or_default();

                let fraction = if total_bytes > 0 {
                    processed_bytes as f32 / total_bytes as f32
                } else {
                    0.0
                };

                egui::Frame::group(ui.style())
                    .inner_margin(egui::Margin::same(12))
                    .show(ui, |ui| {
                        ui.label(
                            egui::RichText::new("Progress")
                                .strong()
                                .size(14.0)
                                .color(ACCENT),
                        );
                        ui.add_space(4.0);

                        let bar = egui::ProgressBar::new(fraction)
                            .text(
                                egui::RichText::new(format!("{:.1}%", fraction * 100.0))
                                    .strong(),
                            )
                            .animate(true);
                        ui.add(bar);

                        ui.add_space(4.0);
                        ui.label(
                            egui::RichText::new(&current_file)
                                .monospace()
                                .size(12.0)
                                .color(TEXT_DIM),
                        );

                        ui.add_space(2.0);
                        ui.label(
                            egui::RichText::new(format!(
                                "{} / {} files    {} / {}",
                                util::format_number(processed_files as u64),
                                util::format_number(total_files as u64),
                                util::format_bytes(processed_bytes),
                                util::format_bytes(total_bytes),
                            ))
                            .color(TEXT_DIM),
                        );
                    });

                ui.ctx().request_repaint();
            }
            AppState::Complete {
                mhl_path,
                checksum_path,
                per_file_count,
            } => {
                egui::Frame::group(ui.style())
                    .inner_margin(egui::Margin::same(12))
                    .show(ui, |ui| {
                        ui.label(
                            egui::RichText::new("Complete")
                                .strong()
                                .size(14.0)
                                .color(SUCCESS),
                        );
                        ui.add_space(6.0);

                        if let Some(path) = mhl_path {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("MHL:").strong());
                                if ui
                                    .link(egui::RichText::new(path).monospace().size(12.0))
                                    .on_hover_text("Click to open")
                                    .clicked()
                                {
                                    open_file(path);
                                }
                            });
                        }
                        if let Some(path) = checksum_path {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("Checksum:").strong());
                                if ui
                                    .link(egui::RichText::new(path).monospace().size(12.0))
                                    .on_hover_text("Click to open")
                                    .clicked()
                                {
                                    open_file(path);
                                }
                            });
                        }
                        if let Some(count) = per_file_count {
                            ui.label(
                                egui::RichText::new(format!(
                                    "Per-file: {} .mash sidecar files written",
                                    count
                                ))
                                .color(TEXT_DIM),
                            );
                        }
                    });
            }
            AppState::Error(msg) => {
                egui::Frame::group(ui.style())
                    .inner_margin(egui::Margin::same(12))
                    .show(ui, |ui| {
                        ui.label(
                            egui::RichText::new("Error")
                                .strong()
                                .size(14.0)
                                .color(ERROR_RED),
                        );
                        ui.add_space(4.0);
                        ui.label(egui::RichText::new(msg).color(ERROR_RED));
                    });
            }
            AppState::Idle => {}
        }
    }
}

fn open_file(path: &str) {
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("explorer").arg(path).spawn();
    }
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open").arg(path).spawn();
    }
    #[cfg(target_os = "linux")]
    {
        let _ = std::process::Command::new("xdg-open").arg(path).spawn();
    }
}
