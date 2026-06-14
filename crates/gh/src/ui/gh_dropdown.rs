use crate::state::AppState;
use eframe::egui;

pub fn show(
    ui: &mut egui::Ui,
    _repo_name: Option<&str>,
    _current_branch: Option<&str>,
    _state: &AppState,
    _current_repo_owned_by_authed_user: Option<bool>,
) {
    // Set popup dimensions
    ui.set_min_size(egui::vec2(640.0, 340.0));
    ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);

        // --- LEFT PANEL (Sidebar for activities) ---
        ui.allocate_ui(egui::vec2(200.0, 340.0), |ui| {
            let rect = ui.max_rect();

            // Round only the left-side corners to align with popup rounding
            let left_rounding = egui::CornerRadius {
                nw: 6,
                ne: 0,
                se: 0,
                sw: 6,
            };
            ui.painter()
                .rect_filled(rect, left_rounding, egui::Color32::from_rgb(28, 28, 28));

            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                // Centered vertically: (340px total height - 20px text height) / 2 = 160px spacer
                ui.add_space(160.0);
                ui.label(
                    egui::RichText::new("No Activities")
                        .color(egui::Color32::from_rgb(140, 140, 140))
                        .size(13.0)
                        .strong(),
                );
            });
        });

        // --- VERTICAL DIVIDER ---
        ui.allocate_ui(egui::vec2(1.0, 340.0), |ui| {
            let rect = ui.max_rect();
            ui.painter()
                .rect_filled(rect, 0.0, egui::Color32::from_rgb(55, 55, 55));
        });

        // --- RIGHT PANEL (Main detail view with logo in the center) ---
        ui.allocate_ui(egui::vec2(439.0, 340.0), |ui| {
            let rect = ui.max_rect();

            // Round only the right-side corners to align with popup rounding
            let right_rounding = egui::CornerRadius {
                nw: 0,
                ne: 6,
                se: 6,
                sw: 0,
            };
            ui.painter()
                .rect_filled(rect, right_rounding, egui::Color32::from_rgb(21, 21, 21));

            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                // Centered vertically: (340px total height - 80px content height) / 2 = 130px spacer
                ui.add_space(130.0);

                // Draw logo image using egui image widget for proper layout centering
                ui.add(
                    egui::Image::new(egui::include_image!("../../../../src/assets/logo.svg"))
                        .max_width(48.0)
                        .max_height(48.0)
                        .tint(egui::Color32::from_rgb(140, 140, 140)),
                );

                ui.add_space(12.0);

                ui.label(
                    egui::RichText::new("No Selected Activity")
                        .color(egui::Color32::from_rgb(140, 140, 140))
                        .size(13.0)
                        .strong(),
                );
            });
        });
    });
}
