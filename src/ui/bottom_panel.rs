use eframe::egui;
use egui_phosphor::regular::{GIT_COMMIT, TERMINAL_WINDOW, X};

use crate::git::models::FileStatus;
use crate::state::AppState;
use crate::ui::commit_drawer;
use crate::ui::terminal_panel;

const BAR_H: f32 = 34.0;
const GRIP_H: f32 = 6.0;
const GRIP_VISUAL_W: f32 = 40.0;
const CLOSE_BTN_SIZE: f32 = 16.0;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BottomTab {
    Terminal,
    CommitDrawer,
}

pub struct BottomPanelState {
    pub active_tab: BottomTab,
    pub transition_progress: f32,
    pub previous_tab: BottomTab,
}

impl Default for BottomPanelState {
    fn default() -> Self {
        Self {
            active_tab: BottomTab::Terminal,
            transition_progress: 1.0,
            previous_tab: BottomTab::Terminal,
        }
    }
}

pub enum BottomPanelResponse {
    None,
    CloseCommitDrawer,
}

pub fn total_height(
    terminal_state: &terminal_panel::State,
    drawer_height: f32,
    commit_open: bool,
) -> f32 {
    let has_terminal = terminal_state.open;
    let has_commit = commit_open;
    if has_terminal && has_commit {
        let content_height = terminal_state.height.max(drawer_height);
        GRIP_H + BAR_H + content_height
    } else if has_terminal {
        terminal_state.height
    } else if has_commit {
        drawer_height
    } else {
        0.0
    }
}

#[allow(clippy::too_many_arguments)]
pub fn show(
    ui: &mut egui::Ui,
    rect: egui::Rect,
    bottom_panel_state: &mut BottomPanelState,
    terminal_state: &mut terminal_panel::State,
    commit_drawer_state: &mut commit_drawer::State,
    app_state: &AppState,
    commit: Option<&commit_drawer::CommitDrawerCommit>,
    signature: Option<&commit_drawer::CommitDrawerSignature>,
    files: &[FileStatus],
    diff: Option<&crate::cdv::CommitDiffViewModel>,
    commit_open: bool,
) -> BottomPanelResponse {
    let has_terminal = terminal_state.open;
    let has_commit = commit_open;

    if !has_terminal && !has_commit {
        return BottomPanelResponse::None;
    }

    if has_terminal && !has_commit {
        if terminal_state.has_pending_spawn() {
            let h = (rect.height() * 0.35).clamp(100.0, 600.0);
            terminal_state.height = h;
        }
        terminal_panel::show(ui, rect, terminal_state);
        return BottomPanelResponse::None;
    }

    if has_commit && !has_terminal {
        let response = commit_drawer::show(
            ui,
            rect,
            commit_drawer_state,
            app_state,
            commit,
            signature,
            files,
            diff,
            false,
            false,
        );
        return match response {
            commit_drawer::CommitDrawerResponse::Close => BottomPanelResponse::CloseCommitDrawer,
            _ => BottomPanelResponse::None,
        };
    }

    let bar_fill = egui::Color32::from_rgb(40, 40, 40);
    let grip_fill = egui::Color32::from_rgb(36, 36, 36);
    let border = egui::Color32::from_rgb(55, 55, 55);
    let active_bg = egui::Color32::from_rgb(58, 58, 58);
    let active_text = egui::Color32::from_rgb(220, 220, 220);
    let inactive_text = egui::Color32::from_rgb(120, 120, 120);

    let terminal_active = bottom_panel_state.active_tab == BottomTab::Terminal;
    let commit_active = bottom_panel_state.active_tab == BottomTab::CommitDrawer;

    bottom_panel_state.transition_progress =
        (bottom_panel_state.transition_progress + ui.input(|i| i.predicted_dt) * 8.0).min(1.0);

    let old_spacing = ui.spacing().item_spacing;
    ui.spacing_mut().item_spacing.y = 0.0;

    let (grip_rect, _) =
        ui.allocate_exact_size(egui::vec2(rect.width(), GRIP_H), egui::Sense::hover());
    let grip_resp = ui.interact(
        grip_rect,
        ui.make_persistent_id("bp_resize"),
        egui::Sense::drag(),
    );
    if grip_resp.dragged() {
        terminal_state.height =
            (terminal_state.height - grip_resp.drag_delta().y).clamp(100.0, 600.0);
        ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeVertical);
    }
    if grip_resp.hovered() || grip_resp.dragged() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeVertical);
    }

    ui.painter().rect_filled(grip_rect, 0.0, grip_fill);
    let grip_line_center_y = grip_rect.center().y;
    let grip_line_left = grip_rect.left() + grip_rect.width() / 2.0 - GRIP_VISUAL_W / 2.0;
    let grip_line_right = grip_rect.left() + grip_rect.width() / 2.0 + GRIP_VISUAL_W / 2.0;
    ui.painter().line_segment(
        [
            egui::pos2(grip_line_left, grip_line_center_y),
            egui::pos2(grip_line_right, grip_line_center_y),
        ],
        egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(80, 80, 80)),
    );
    ui.painter().line_segment(
        [grip_rect.left_bottom(), grip_rect.right_bottom()],
        egui::Stroke::new(1.0_f32, border),
    );

    let (bar_rect, _) =
        ui.allocate_exact_size(egui::vec2(rect.width(), BAR_H), egui::Sense::hover());

    ui.painter().rect_filled(bar_rect, 0.0, bar_fill);
    ui.painter().line_segment(
        [bar_rect.left_bottom(), bar_rect.right_bottom()],
        egui::Stroke::new(1.0_f32, border),
    );
    ui.painter().line_segment(
        [bar_rect.left_top(), bar_rect.right_top()],
        egui::Stroke::new(1.0_f32, border),
    );

    let mut x = bar_rect.left();
    let btn_y = bar_rect.center().y;

    let term_label = format!("{} Terminal", TERMINAL_WINDOW);
    let commit_label = format!("{} Commit", GIT_COMMIT);

    for (label, is_active, tab, id) in [
        (
            term_label.as_str(),
            terminal_active,
            BottomTab::Terminal,
            "bp_tab_terminal",
        ),
        (
            commit_label.as_str(),
            commit_active,
            BottomTab::CommitDrawer,
            "bp_tab_commit",
        ),
    ] {
        let text_color = if is_active {
            active_text
        } else {
            inactive_text
        };

        let icon_galley = ui.painter().layout_no_wrap(
            label.chars().next().unwrap_or(' ').to_string(),
            egui::FontId::proportional(12.0),
            text_color,
        );
        let label_text: String = label.chars().skip(1).collect();
        let label_galley = ui.painter().layout_no_wrap(
            label_text.trim().to_string(),
            egui::FontId::proportional(10.0),
            text_color,
        );

        let close_w = if is_active {
            CLOSE_BTN_SIZE + 10.0
        } else {
            0.0
        };
        let btn_w = icon_galley.size().x + label_galley.size().x + 16.0 + close_w;
        let btn_h = BAR_H - 6.0;
        let btn_rect = egui::Rect::from_center_size(
            egui::pos2(x + btn_w / 2.0, btn_y),
            egui::vec2(btn_w, btn_h),
        );

        let resp = ui.interact(btn_rect, ui.make_persistent_id(id), egui::Sense::click());

        if is_active {
            ui.painter().rect_filled(btn_rect, 0.0, active_bg);
        } else if resp.hovered() {
            ui.painter()
                .rect_filled(btn_rect, 0.0, egui::Color32::from_rgb(50, 50, 50));
        }

        if resp.clicked() {
            if bottom_panel_state.active_tab != tab {
                bottom_panel_state.previous_tab = bottom_panel_state.active_tab;
                bottom_panel_state.transition_progress = 0.0;
            }
            bottom_panel_state.active_tab = tab;
        }

        let icon_y = btn_rect.center().y - icon_galley.size().y / 2.0;
        let label_y = btn_rect.center().y - label_galley.size().y / 2.0;
        let icon_x = btn_rect.left() + 8.0;
        let label_x = icon_x + icon_galley.size().x + 6.0;

        ui.painter()
            .galley(egui::pos2(icon_x, icon_y), icon_galley, text_color);
        ui.painter()
            .galley(egui::pos2(label_x, label_y), label_galley, text_color);

        if is_active {
            let close_x = btn_rect.right() - CLOSE_BTN_SIZE - 4.0;
            let close_rect = egui::Rect::from_center_size(
                egui::pos2(close_x, btn_rect.center().y),
                egui::vec2(CLOSE_BTN_SIZE, CLOSE_BTN_SIZE),
            );
            let close_resp = ui.interact(
                close_rect,
                ui.make_persistent_id(format!("{}_close", id)),
                egui::Sense::click(),
            );
            let close_color = if close_resp.hovered() {
                egui::Color32::from_rgb(230, 80, 80)
            } else {
                egui::Color32::from_rgb(160, 160, 160)
            };
            ui.painter().text(
                close_rect.center(),
                egui::Align2::CENTER_CENTER,
                X,
                egui::FontId::proportional(10.0),
                close_color,
            );
            if close_resp.clicked() {
                match tab {
                    BottomTab::Terminal => {
                        terminal_state.close();
                    }
                    BottomTab::CommitDrawer => {
                        bottom_panel_state.active_tab = BottomTab::Terminal;
                        ui.spacing_mut().item_spacing = old_spacing;
                        return BottomPanelResponse::CloseCommitDrawer;
                    }
                }
            }
        }

        x += btn_w;
    }

    ui.spacing_mut().item_spacing = old_spacing;

    let content_rect = egui::Rect::from_min_max(
        egui::pos2(rect.left(), bar_rect.bottom()),
        rect.right_bottom(),
    );

    let t = egui::remap(bottom_panel_state.transition_progress, 0.0..=1.0, 0.0..=1.0);
    let alpha = (t * 255.0) as u8;
    let content_bg = egui::Color32::from_rgba_unmultiplied(30, 30, 30, alpha);

    ui.painter().rect_filled(content_rect, 0.0, content_bg);

    match bottom_panel_state.active_tab {
        BottomTab::Terminal => {
            if terminal_state.has_pending_spawn() {
                let h = (content_rect.height() * 0.35).clamp(100.0, 600.0);
                terminal_state.height = h;
            }
            terminal_panel::show_headerless(ui, content_rect, terminal_state);
            BottomPanelResponse::None
        }
        BottomTab::CommitDrawer => {
            let response = commit_drawer::show(
                ui,
                content_rect,
                commit_drawer_state,
                app_state,
                commit,
                signature,
                files,
                diff,
                false,
                true,
            );
            match response {
                commit_drawer::CommitDrawerResponse::Close => {
                    BottomPanelResponse::CloseCommitDrawer
                }
                _ => BottomPanelResponse::None,
            }
        }
    }
}
