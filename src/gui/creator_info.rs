use crate::app::MashApp;

const ACCENT: egui::Color32 = egui::Color32::from_rgb(80, 160, 255);
const TEXT_DIM: egui::Color32 = egui::Color32::from_rgb(160, 160, 170);

impl MashApp {
    pub fn creator_info_panel(&mut self, ui: &mut egui::Ui) {
        egui::Frame::group(ui.style())
            .inner_margin(egui::Margin::same(12))
            .show(ui, |ui| {
                egui::CollapsingHeader::new(
                    egui::RichText::new("Creator Info")
                        .strong()
                        .size(14.0)
                        .color(ACCENT),
                )
                .default_open(false)
                .show(ui, |ui| {
                    ui.add_space(4.0);
                    ui.label(
                        egui::RichText::new("Optional metadata for the MHL manifest")
                            .size(12.0)
                            .color(TEXT_DIM),
                    );
                    ui.add_space(6.0);

                    egui::Grid::new("creator_info_grid")
                        .num_columns(2)
                        .spacing([12.0, 6.0])
                        .min_col_width(70.0)
                        .show(ui, |ui| {
                            ui.label("Author:");
                            ui.add(
                                egui::TextEdit::singleline(&mut self.author_name)
                                    .desired_width(f32::INFINITY)
                                    .hint_text("Name"),
                            );
                            ui.end_row();

                            ui.label("Email:");
                            ui.add(
                                egui::TextEdit::singleline(&mut self.author_email)
                                    .desired_width(f32::INFINITY)
                                    .hint_text("email@example.com"),
                            );
                            ui.end_row();

                            ui.label("Location:");
                            ui.add(
                                egui::TextEdit::singleline(&mut self.location)
                                    .desired_width(f32::INFINITY)
                                    .hint_text("Studio / City"),
                            );
                            ui.end_row();

                            ui.label("Comment:");
                            ui.add(
                                egui::TextEdit::singleline(&mut self.comment)
                                    .desired_width(f32::INFINITY)
                                    .hint_text("Notes"),
                            );
                            ui.end_row();
                        });
                });
            });
    }
}
