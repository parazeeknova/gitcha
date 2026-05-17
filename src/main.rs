use eframe::egui;

mod body;
mod commit_panel;
mod sidebar;
mod tabbar;
mod titlebar;
mod toolbar;

fn main() -> eframe::Result {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Palimpsest")
            .with_inner_size([960.0, 720.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Palimpsest",
        native_options,
        Box::new(|cc| Ok(Box::new(PalimpsestApp::new(cc)))),
    )
}

struct PalimpsestApp {
    titlebar_menu_open: bool,
    search_query: String,
    body_state: body::State,
    commit_panel_state: commit_panel::State,
}

impl PalimpsestApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut fonts = egui::FontDefinitions::default();
        egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);
        cc.egui_ctx.set_fonts(fonts);
        egui_extras::install_image_loaders(&cc.egui_ctx);

        Self {
            titlebar_menu_open: false,
            search_query: String::new(),
            body_state: body::State::default(),
            commit_panel_state: commit_panel::State::default(),
        }
    }
}

impl eframe::App for PalimpsestApp {
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        let background = ui.visuals().widgets.inactive.bg_fill;
        ui.painter().rect_filled(ui.max_rect(), 0.0, background);

        titlebar::show(
            ui,
            frame,
            &mut self.titlebar_menu_open,
            &mut self.search_query,
        );
        toolbar::show(ui);
        tabbar::show(ui);

        let content_rect = ui.available_rect_before_wrap();
        let (content_rect, _) = ui.allocate_exact_size(content_rect.size(), egui::Sense::hover());
        let sidebar_rect = egui::Rect::from_min_size(
            content_rect.left_top(),
            egui::vec2(sidebar::SIDEBAR_WIDTH, content_rect.height()),
        );
        let body_rect = egui::Rect::from_min_max(
            egui::pos2(sidebar_rect.right(), content_rect.top()),
            content_rect.right_bottom(),
        );

        ui.scope_builder(
            egui::UiBuilder::new()
                .id_salt("app_sidebar")
                .max_rect(sidebar_rect)
                .layout(egui::Layout::top_down(egui::Align::Min)),
            sidebar::show,
        );
        ui.scope_builder(
            egui::UiBuilder::new()
                .id_salt("app_body")
                .max_rect(body_rect)
                .layout(egui::Layout::top_down(egui::Align::Min)),
            |ui| body::show(ui, &mut self.body_state, &mut self.commit_panel_state),
        );
    }
}
