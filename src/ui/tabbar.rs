use eframe::egui;
use egui_phosphor::regular::{PLUS, X};

use crate::state::AppState;

const TABBAR_HEIGHT: f32 = 25.0;
const PLUS_WIDTH: f32 = 28.0;
const CLOSE_WIDTH: f32 = 18.0;

pub enum TabAction {
    Open,
    Activate(usize),
    Close(usize),
}

struct Tab<'a> {
    title: &'a str,
    active: bool,
    closeable: bool,
}

pub fn show(ui: &mut egui::Ui, state: &AppState) -> Option<TabAction> {
    let width = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width, TABBAR_HEIGHT), egui::Sense::hover());

    let stroke = egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(72, 72, 72));
    let top_stroke = egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(88, 88, 88));
    let bg_fill = egui::Color32::from_rgb(34, 34, 34);
    ui.painter().rect_filled(rect, 0.0, bg_fill);
    ui.painter()
        .line_segment([rect.left_top(), rect.right_top()], top_stroke);
    ui.painter()
        .line_segment([rect.left_bottom(), rect.right_bottom()], stroke);

    let tabs: Vec<Tab<'_>> = state
        .open_tabs
        .iter()
        .enumerate()
        .map(|(index, path)| Tab {
            title: repo_display_name(path),
            active: state.active_tab == Some(index),
            closeable: true,
        })
        .collect();

    let plus_rect = egui::Rect::from_min_max(
        egui::pos2(rect.right() - PLUS_WIDTH, rect.top()),
        rect.right_bottom(),
    );
    let tabs_rect = egui::Rect::from_min_max(rect.left_top(), plus_rect.left_bottom());

    if tabs.is_empty() {
        paint_plus(ui, plus_rect, stroke);
        if ui
            .interact(
                plus_rect,
                ui.make_persistent_id("tabbar_open"),
                egui::Sense::click(),
            )
            .clicked()
        {
            return Some(TabAction::Open);
        }
        return None;
    }

    let mut left = tabs_rect.left();
    for (index, tab) in tabs.iter().enumerate() {
        if left >= tabs_rect.right() {
            break;
        }

        let width = if index == 0 {
            tabs_rect.width().min(240.0)
        } else {
            let remaining_tabs = (tabs.len() - index) as f32;
            ((tabs_rect.right() - left) / remaining_tabs).max(112.0)
        };
        let right = (left + width).min(tabs_rect.right());
        let tab_rect = egui::Rect::from_min_max(
            egui::pos2(left, tabs_rect.top()),
            egui::pos2(right, tabs_rect.bottom()),
        );
        if let Some(action) = paint_tab(ui, tab_rect, tab, index, stroke) {
            return Some(action);
        }
        left = right;
    }

    paint_plus(ui, plus_rect, stroke);
    if ui
        .interact(
            plus_rect,
            ui.make_persistent_id("tabbar_open"),
            egui::Sense::click(),
        )
        .clicked()
    {
        return Some(TabAction::Open);
    }

    None
}

fn paint_tab(
    ui: &mut egui::Ui,
    rect: egui::Rect,
    tab: &Tab<'_>,
    index: usize,
    stroke: egui::Stroke,
) -> Option<TabAction> {
    let fill = if tab.active {
        egui::Color32::from_rgb(62, 62, 62)
    } else {
        egui::Color32::from_rgb(38, 38, 38)
    };

    ui.painter().rect_filled(rect, 0.0, fill);
    ui.painter()
        .line_segment([rect.right_top(), rect.right_bottom()], stroke);

    let title_rect = rect.shrink2(egui::vec2(8.0, 0.0));
    let close_rect = egui::Rect::from_min_size(
        egui::pos2(rect.right() - CLOSE_WIDTH, rect.center().y - 8.0),
        egui::vec2(CLOSE_WIDTH, 16.0),
    );

    ui.painter().text(
        title_rect.left_center(),
        egui::Align2::LEFT_CENTER,
        tab.title,
        egui::FontId::proportional(13.0),
        ui.visuals().text_color(),
    );

    let activate_response = ui.interact(
        rect,
        ui.make_persistent_id(("tabbar_tab", index)),
        egui::Sense::click(),
    );
    if activate_response.clicked() {
        return Some(TabAction::Activate(index));
    }

    if tab.closeable {
        ui.painter().text(
            close_rect.center(),
            egui::Align2::CENTER_CENTER,
            X,
            egui::FontId::proportional(10.0),
            ui.visuals().text_color(),
        );
        let close_response = ui.interact(
            close_rect,
            ui.make_persistent_id(("tabbar_close", index)),
            egui::Sense::click(),
        );
        if close_response.clicked() {
            return Some(TabAction::Close(index));
        }
    }

    None
}

fn paint_plus(ui: &mut egui::Ui, rect: egui::Rect, stroke: egui::Stroke) {
    ui.painter()
        .line_segment([rect.left_top(), rect.left_bottom()], stroke);
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        PLUS,
        egui::FontId::proportional(20.0),
        ui.visuals().text_color(),
    );
}

fn repo_display_name(path: &str) -> &str {
    std::path::Path::new(path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(path)
}
