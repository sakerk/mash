use crate::checksum_file;
use crate::diff;
use crate::gui::compare_panel::DiffFilter;
use crate::hashing::engine::{HashJob, HashResult};
use crate::manifest_reader;
use crate::mhl;
use crate::mhl::types::*;
use std::path::PathBuf;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use walkdir::WalkDir;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppMode {
    Hash,
    Compare,
}

pub enum AppState {
    Idle,
    Hashing {
        job: Arc<HashJob>,
    },
    Complete {
        mhl_path: Option<String>,
        checksum_path: Option<String>,
        per_file_count: Option<u64>,
    },
    Error(String),
}

pub struct MashApp {
    // Folder selection
    pub folder_path: String,
    pub folder_scanned: bool,
    pub file_count: u64,
    pub total_size: u64,

    // Algorithm selection
    pub selected_algorithm: HashAlgorithm,

    // Creator info
    pub author_name: String,
    pub author_email: String,
    pub location: String,
    pub comment: String,

    // Checksum file config
    pub checksum_separator: char,
    pub selected_columns: Vec<ChecksumColumn>,

    // Output options
    pub generate_mhl: bool,
    pub generate_checksum: bool,
    pub generate_per_file: bool,

    // Per-file .mash config
    pub per_file_columns: Vec<ChecksumColumn>,
    pub per_file_separator: char,
    pub per_file_header: bool,
    pub per_file_extension: String,

    // State
    pub state: AppState,

    // Help window
    pub show_help: bool,

    // Mode
    pub mode: AppMode,

    // Compare mode
    pub compare_file_a: String,
    pub compare_file_b: String,
    pub compare_result: Option<DiffResult>,
    pub compare_error: Option<String>,
    pub compare_filter: DiffFilter,
}

impl Default for MashApp {
    fn default() -> Self {
        Self {
            folder_path: String::new(),
            folder_scanned: false,
            file_count: 0,
            total_size: 0,
            selected_algorithm: HashAlgorithm::Xxh64,
            author_name: String::new(),
            author_email: String::new(),
            location: String::new(),
            comment: String::new(),
            checksum_separator: ',',
            selected_columns: vec![
                ChecksumColumn::Checksum,
                ChecksumColumn::FilePath,
                ChecksumColumn::Algorithm,
            ],
            generate_mhl: true,
            generate_checksum: true,
            generate_per_file: false,
            per_file_columns: vec![
                ChecksumColumn::Checksum,
                ChecksumColumn::FilePath,
                ChecksumColumn::Algorithm,
            ],
            per_file_separator: ',',
            per_file_header: false,
            per_file_extension: "mash".to_string(),
            state: AppState::Idle,
            show_help: false,
            mode: AppMode::Hash,
            compare_file_a: String::new(),
            compare_file_b: String::new(),
            compare_result: None,
            compare_error: None,
            compare_filter: DiffFilter::All,
        }
    }
}

impl MashApp {
    pub fn scan_folder(&mut self) {
        let path = PathBuf::from(&self.folder_path);

        if path.is_file() {
            self.file_count = 1;
            self.total_size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
            self.folder_scanned = true;
            return;
        }

        if !path.is_dir() {
            self.folder_scanned = false;
            return;
        }

        let mut count = 0u64;
        let mut size = 0u64;

        for entry in WalkDir::new(&path)
            .into_iter()
            .filter_entry(|e| {
                let name = e.file_name().to_string_lossy();
                !(e.file_type().is_dir() && name == "ascmhl")
            })
            .flatten()
        {
            if entry.file_type().is_file() {
                count += 1;
                if let Ok(meta) = entry.metadata() {
                    size += meta.len();
                }
            }
        }

        self.file_count = count;
        self.total_size = size;
        self.folder_scanned = true;
    }

    fn start_hashing(&mut self) {
        let target = PathBuf::from(&self.folder_path);
        if !target.is_dir() && !target.is_file() {
            self.state = AppState::Error("Invalid path".to_string());
            return;
        }

        let job = Arc::new(HashJob::new());
        let algorithm = self.selected_algorithm;
        let job_clone = job.clone();

        self.state = AppState::Hashing { job: job.clone() };

        std::thread::spawn(move || {
            crate::hashing::engine::run_hashing(target, vec![algorithm], Vec::new(), job_clone);
        });
    }

    fn cancel_hashing(&mut self) {
        if let AppState::Hashing { ref job, .. } = self.state {
            job.cancel.store(true, Ordering::Relaxed);
        }
    }

    fn check_hashing_complete(&mut self) {
        let result = if let AppState::Hashing { ref job, .. } = self.state {
            let progress = &job.progress;
            if let Ok(result_lock) = progress.result.lock() {
                result_lock.clone()
            } else {
                None
            }
        } else {
            return;
        };

        if let Some(hash_result) = result {
            match hash_result {
                Ok(result) => self.finalize_output(result),
                Err(e) => {
                    self.state = AppState::Error(e);
                }
            }
        }
    }

    fn finalize_output(&mut self, result: HashResult) {
        let mut mhl_path = None;
        let mut checksum_path = None;
        let mut per_file_count = None;

        if self.generate_mhl {
            let author_info = if !self.author_name.is_empty() {
                Some(AuthorInfo {
                    name: self.author_name.clone(),
                    email: if self.author_email.is_empty() {
                        None
                    } else {
                        Some(self.author_email.clone())
                    },
                    phone: None,
                    role: None,
                })
            } else {
                None
            };

            let loc = if self.location.is_empty() {
                None
            } else {
                Some(self.location.clone())
            };
            let cmt = if self.comment.is_empty() {
                None
            } else {
                Some(self.comment.clone())
            };

            match mhl::generate_mhl(
                &result,
                &[self.selected_algorithm],
                author_info,
                loc,
                cmt,
                Vec::new(),
            ) {
                Ok(path) => mhl_path = Some(path),
                Err(e) => {
                    self.state = AppState::Error(format!("MHL error: {}", e));
                    return;
                }
            }
        }

        if self.generate_checksum {
            let config = ChecksumFileConfig {
                columns: self.selected_columns.clone(),
                separator: self.checksum_separator,
            };
            let filename =
                checksum_file::checksum_filename(&result.root_path, self.checksum_separator);
            let output_path = result.root_path.join(&filename);

            match checksum_file::write_checksum_file(&result, &config, &output_path) {
                Ok(()) => checksum_path = Some(output_path.to_string_lossy().to_string()),
                Err(e) => {
                    self.state = AppState::Error(format!("Checksum file error: {}", e));
                    return;
                }
            }
        }

        if self.generate_per_file {
            let config = ChecksumFileConfig {
                columns: self.per_file_columns.clone(),
                separator: self.per_file_separator,
            };

            match checksum_file::write_per_file_checksums(&result, &config, self.per_file_header, &self.per_file_extension) {
                Ok(count) => per_file_count = Some(count),
                Err(e) => {
                    self.state = AppState::Error(format!("Per-file checksum error: {}", e));
                    return;
                }
            }
        }

        self.state = AppState::Complete {
            mhl_path,
            checksum_path,
            per_file_count,
        };
    }

    pub fn load_and_compare(&mut self) {
        self.compare_error = None;
        self.compare_result = None;

        let path_a = PathBuf::from(&self.compare_file_a);
        let path_b = PathBuf::from(&self.compare_file_b);

        let manifest_a = match manifest_reader::read_manifest(&path_a) {
            Ok(m) => m,
            Err(e) => {
                self.compare_error = Some(format!("File A: {}", e));
                return;
            }
        };

        let manifest_b = match manifest_reader::read_manifest(&path_b) {
            Ok(m) => m,
            Err(e) => {
                self.compare_error = Some(format!("File B: {}", e));
                return;
            }
        };

        self.compare_result = Some(diff::compare_manifests(&manifest_a, &manifest_b));
        self.compare_filter = DiffFilter::All;
    }
}

// -- Theming --

const ACCENT: egui::Color32 = egui::Color32::from_rgb(80, 160, 255);
const ACCENT_DARK: egui::Color32 = egui::Color32::from_rgb(50, 120, 210);
const BG_PANEL: egui::Color32 = egui::Color32::from_rgb(30, 30, 34);
const BG_WIDGET: egui::Color32 = egui::Color32::from_rgb(42, 42, 48);
const TEXT_DIM: egui::Color32 = egui::Color32::from_rgb(160, 160, 170);

fn apply_theme(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();

    // Spacing
    style.spacing.item_spacing = egui::vec2(8.0, 6.0);
    style.spacing.button_padding = egui::vec2(12.0, 6.0);
    style.spacing.window_margin = egui::Margin::same(16);

    // Rounding
    let rounding = egui::CornerRadius::same(6);
    style.visuals.widgets.noninteractive.corner_radius = rounding;
    style.visuals.widgets.inactive.corner_radius = rounding;
    style.visuals.widgets.hovered.corner_radius = rounding;
    style.visuals.widgets.active.corner_radius = rounding;
    style.visuals.window_corner_radius = egui::CornerRadius::same(8);

    // Dark theme base
    style.visuals.dark_mode = true;
    style.visuals.panel_fill = BG_PANEL;
    style.visuals.window_fill = BG_PANEL;
    style.visuals.extreme_bg_color = BG_WIDGET;

    // Widget backgrounds
    style.visuals.widgets.noninteractive.bg_fill = BG_WIDGET;
    style.visuals.widgets.inactive.bg_fill = BG_WIDGET;
    style.visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(55, 55, 62);
    style.visuals.widgets.active.bg_fill = ACCENT_DARK;

    // Widget strokes
    style.visuals.widgets.noninteractive.bg_stroke =
        egui::Stroke::new(1.0, egui::Color32::from_rgb(60, 60, 68));
    style.visuals.widgets.inactive.bg_stroke =
        egui::Stroke::new(1.0, egui::Color32::from_rgb(70, 70, 80));
    style.visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, ACCENT);

    // Selection
    style.visuals.selection.bg_fill = ACCENT_DARK.linear_multiply(0.4);
    style.visuals.selection.stroke = egui::Stroke::new(1.0, ACCENT);

    ctx.set_style(style);
}

fn section_heading(ui: &mut egui::Ui, label: &str) {
    ui.add_space(2.0);
    ui.horizontal(|ui| {
        ui.add_space(2.0);
        ui.label(egui::RichText::new(label).strong().size(14.0).color(ACCENT));
    });
    ui.add_space(2.0);
}

impl eframe::App for MashApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        apply_theme(ctx);
        self.check_hashing_complete();

        // Top bar
        egui::TopBottomPanel::top("header").show(ctx, |ui| {
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.add_space(8.0);
                ui.label(
                    egui::RichText::new("MASH")
                        .strong()
                        .size(20.0)
                        .color(ACCENT),
                );
                ui.separator();

                // Mode tabs
                let hash_text = if self.mode == AppMode::Hash {
                    egui::RichText::new("Hash").strong().color(ACCENT)
                } else {
                    egui::RichText::new("Hash").color(TEXT_DIM)
                };
                if ui.add(egui::Button::new(hash_text).frame(false)).clicked() {
                    self.mode = AppMode::Hash;
                }

                let compare_text = if self.mode == AppMode::Compare {
                    egui::RichText::new("Compare").strong().color(ACCENT)
                } else {
                    egui::RichText::new("Compare").color(TEXT_DIM)
                };
                if ui.add(egui::Button::new(compare_text).frame(false)).clicked() {
                    self.mode = AppMode::Compare;
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .add(egui::Button::new(
                            egui::RichText::new("?").strong().size(14.0),
                        ).min_size(egui::vec2(28.0, 28.0)))
                        .on_hover_text("Help")
                        .clicked()
                    {
                        self.show_help = !self.show_help;
                    }
                    ui.label(
                        egui::RichText::new(format!("v{}", env!("CARGO_PKG_VERSION")))
                            .size(11.0)
                            .color(TEXT_DIM),
                    );
                });
            });
            ui.add_space(4.0);
        });

        // Help window
        if self.show_help {
            self.help_window(ctx);
        }

        // Bottom action bar
        egui::TopBottomPanel::bottom("actions").show(ctx, |ui| {
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.add_space(8.0);

                match self.mode {
                    AppMode::Hash => {
                        let is_hashing = matches!(self.state, AppState::Hashing { .. });
                        let can_start = !self.folder_path.is_empty()
                            && !is_hashing;

                        let start_btn = egui::Button::new(
                            egui::RichText::new("Start Hashing").strong().size(14.0),
                        )
                        .min_size(egui::vec2(140.0, 36.0))
                        .fill(if can_start { ACCENT_DARK } else { BG_WIDGET });

                        if ui.add_enabled(can_start, start_btn).clicked() {
                            self.start_hashing();
                        }

                        if is_hashing {
                            let cancel_btn = egui::Button::new(
                                egui::RichText::new("Cancel").size(14.0),
                            )
                            .min_size(egui::vec2(100.0, 36.0));
                            if ui.add(cancel_btn).clicked() {
                                self.cancel_hashing();
                            }
                        }

                        if matches!(self.state, AppState::Complete { .. } | AppState::Error(_)) {
                            let reset_btn = egui::Button::new(
                                egui::RichText::new("Reset").size(14.0),
                            )
                            .min_size(egui::vec2(100.0, 36.0));
                            if ui.add(reset_btn).clicked() {
                                self.state = AppState::Idle;
                            }
                        }
                    }
                    AppMode::Compare => {
                        let can_compare = !self.compare_file_a.is_empty()
                            && !self.compare_file_b.is_empty();

                        let compare_btn = egui::Button::new(
                            egui::RichText::new("Compare").strong().size(14.0),
                        )
                        .min_size(egui::vec2(140.0, 36.0))
                        .fill(if can_compare { ACCENT_DARK } else { BG_WIDGET });

                        if ui.add_enabled(can_compare, compare_btn).clicked() {
                            self.load_and_compare();
                        }

                        if self.compare_result.is_some() || self.compare_error.is_some() {
                            let reset_btn = egui::Button::new(
                                egui::RichText::new("Reset").size(14.0),
                            )
                            .min_size(egui::vec2(100.0, 36.0));
                            if ui.add(reset_btn).clicked() {
                                self.compare_result = None;
                                self.compare_error = None;
                            }
                        }
                    }
                }
            });
            ui.add_space(8.0);
        });

        // Main content
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    match self.mode {
                        AppMode::Hash => {
                            ui.add_space(4.0);
                            self.folder_panel(ui);
                            ui.add_space(6.0);
                            self.algorithm_panel(ui);
                            ui.add_space(6.0);
                            self.output_toggles_panel(ui);
                            if self.generate_mhl {
                                ui.add_space(6.0);
                                self.creator_info_panel(ui);
                            }
                            if self.generate_checksum || self.generate_per_file {
                                ui.add_space(6.0);
                                self.checksum_settings_panel(ui);
                            }
                            ui.add_space(6.0);
                            self.progress_panel(ui);
                            ui.add_space(8.0);
                        }
                        AppMode::Compare => {
                            ui.add_space(4.0);
                            self.compare_panel(ui);
                            ui.add_space(8.0);
                        }
                    }
                });
        });
    }
}

impl MashApp {
    fn help_window(&mut self, ctx: &egui::Context) {
        egui::Window::new("Help")
            .open(&mut self.show_help)
            .resizable(true)
            .default_width(520.0)
            .show(ctx, |ui| {
                ui.label(
                    egui::RichText::new("MASH")
                        .strong()
                        .size(18.0)
                        .color(ACCENT),
                );
                ui.label(
                    egui::RichText::new("Media Asset Hash")
                        .color(TEXT_DIM),
                );
                ui.label(format!("Version {}", env!("CARGO_PKG_VERSION")));
                ui.separator();

                ui.label("MASH hashes media files and generates ASC MHL v2.0 manifests and checksum files (CSV/TSV).");
                ui.add_space(8.0);

                section_heading(ui, "Quick Start");
                ui.label("1. Browse to a folder or type a path");
                ui.label("2. Choose hash algorithms (XXH64 is default)");
                ui.label("3. Optionally fill in Creator Info");
                ui.label("4. Configure checksum columns and format");
                ui.label("5. Click Start Hashing");
                ui.label("6. Click output file paths to open them");
                ui.add_space(8.0);

                section_heading(ui, "Algorithms");
                egui::Grid::new("help_algos")
                    .num_columns(2)
                    .spacing([20.0, 4.0])
                    .striped(true)
                    .show(ui, |ui| {
                        ui.strong("Name");
                        ui.strong("Notes");
                        ui.end_row();
                        ui.label("XXH64");
                        ui.label("64-bit xxHash - fastest, recommended");
                        ui.end_row();
                        ui.label("XXH128");
                        ui.label("128-bit xxHash");
                        ui.end_row();
                        ui.label("XXH3");
                        ui.label("XXH3 64-bit variant");
                        ui.end_row();
                        ui.label("MD5");
                        ui.label("Legacy compatibility");
                        ui.end_row();
                        ui.label("SHA-1");
                        ui.label("Legacy compatibility");
                        ui.end_row();
                    });
                ui.add_space(8.0);

                section_heading(ui, "Output");
                ui.label("MHL  - ASC MHL v2.0 XML manifest in ascmhl/ subfolder");
                ui.label("CSV/TSV - Checksum file with configurable columns");
                ui.add_space(8.0);

                section_heading(ui, "CLI Mode");
                ui.label("Run with arguments for headless operation:");
                ui.add_space(2.0);
                ui.code("mash -d <FOLDER> [OPTIONS]");
                ui.code("mash --help");
            });
    }
}

fn load_icon() -> Option<egui::IconData> {
    let png_bytes = include_bytes!("../assets/icon.png");
    let img = image::load_from_memory(png_bytes).ok()?.into_rgba8();
    let (w, h) = img.dimensions();
    Some(egui::IconData {
        rgba: img.into_raw(),
        width: w,
        height: h,
    })
}

pub fn run_gui() {
    let mut viewport = egui::ViewportBuilder::default()
        .with_inner_size([640.0, 740.0])
        .with_min_inner_size([520.0, 480.0])
        .with_title("MASH");

    if let Some(icon) = load_icon() {
        viewport = viewport.with_icon(std::sync::Arc::new(icon));
    }

    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "MASH",
        options,
        Box::new(|cc| {
            // Set default font sizes slightly larger
            let mut style = (*cc.egui_ctx.style()).clone();
            style.text_styles.insert(
                egui::TextStyle::Body,
                egui::FontId::proportional(14.0),
            );
            style.text_styles.insert(
                egui::TextStyle::Button,
                egui::FontId::proportional(14.0),
            );
            style.text_styles.insert(
                egui::TextStyle::Heading,
                egui::FontId::proportional(18.0),
            );
            style.text_styles.insert(
                egui::TextStyle::Monospace,
                egui::FontId::monospace(13.0),
            );
            cc.egui_ctx.set_style(style);
            Ok(Box::new(MashApp::default()))
        }),
    )
    .expect("Failed to launch GUI");
}
