use eframe::egui;
use egui_phosphor::regular::{
    ARROW_DOWN, CARET_DOWN, CARET_RIGHT, CARET_UP, FILE, FILE_PLUS, FOLDER, GIT_BRANCH, GIT_COMMIT,
    LIST_CHECKS, MINUS, PLUS, TRASH, WARNING, X,
};

use crate::git::GitRepo;
use crate::state::{AppState, CachedFileChangeKind, CachedFileStatus, CommitAction, StashAction};

const PANEL_WIDTH: f32 = 360.0;
const PANEL_HEIGHT: f32 = 520.0;
const PANEL_MARGIN: f32 = 18.0;
const FILE_ROW_HEIGHT: f32 = 22.0;
const MAX_TITLE_LEN: usize = 150;

const HEADER_H: f32 = 32.0;
const MSG_BOX_H: f32 = 80.0;
const BTN_H: f32 = 28.0;
const CONTENT_PAD: f32 = 10.0;
const SECTION_GAP: f32 = 6.0;

fn section_divider(ui: &mut egui::Ui) {
    let (rect, _) =
        ui.allocate_exact_size(egui::vec2(ui.available_width(), 1.0), egui::Sense::hover());
    ui.painter().line_segment(
        [rect.left_center(), rect.right_center()],
        egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(68, 68, 68)),
    );
}

#[derive(Default)]
pub struct State {
    pub title: String,
    pub description: String,
    pub amend: bool,
    pub sign_off: bool,
    pub pending_actions: Vec<CommitAction>,
    pub show_discard_confirm: bool,
    pub collapsed: bool,
    pub options_expanded: bool,
    pub stash_mode: bool,
    pub msg_expanded: bool,
    pub skip_hooks: bool,
    pub pending_stash_action: Option<StashAction>,
}

impl State {
    fn queue_action(&mut self, action: CommitAction) {
        self.pending_actions.push(action);
    }
}

pub fn show(
    ui: &mut egui::Ui,
    body_rect: egui::Rect,
    state: &mut State,
    git_repo: Option<&GitRepo>,
) {
    let status = git_repo.and_then(|r| r.status().ok());
    let header_text = git_repo
        .and_then(|r| r.head_branch().ok())
        .unwrap_or_else(|| "HEAD".to_string());

    let panel_rect = calc_panel_rect(body_rect);

    render_panel(ui, panel_rect, state, header_text.as_str(), &status);
}

pub fn show_cached(
    ui: &mut egui::Ui,
    body_rect: egui::Rect,
    state: &mut State,
    app_state: &AppState,
) {
    let header_text = app_state
        .cached_status
        .as_ref()
        .map(|s| s.branch.clone())
        .unwrap_or_else(|| "HEAD".to_string());

    let panel_rect = calc_panel_rect(body_rect);

    render_panel_cached(ui, panel_rect, state, &header_text, app_state);
}

pub fn show_cached_with_bottom_offset(
    ui: &mut egui::Ui,
    body_rect: egui::Rect,
    bottom_offset: f32,
    state: &mut State,
    app_state: &AppState,
) {
    let header_text = app_state
        .cached_status
        .as_ref()
        .map(|s| s.branch.clone())
        .unwrap_or_else(|| "HEAD".to_string());

    let mut panel_rect = calc_panel_rect(body_rect);
    if bottom_offset > 0.0 {
        panel_rect = panel_rect.translate(egui::vec2(0.0, -bottom_offset));
    }

    let allowed_top = body_rect.top();
    if panel_rect.top() < allowed_top {
        let diff = allowed_top - panel_rect.top();
        panel_rect = panel_rect.translate(egui::vec2(0.0, diff));
    }

    render_panel_cached(ui, panel_rect, state, &header_text, app_state);
}

#[allow(clippy::too_many_arguments)]
pub fn show_selected_commit(
    ui: &mut egui::Ui,
    body_rect: egui::Rect,
    bottom_offset: f32,
    state: &mut State,
    commit_subject: &str,
    commit_hash: &str,
    commit_message: &str,
    files: &[crate::git::models::FileStatus],
) {
    let available_height = (body_rect.height() - PANEL_MARGIN * 2.0).max(0.0);
    let file_count = files.len().max(1);
    let msg_height = if commit_message.is_empty() { 0.0 } else { 80.0 };
    let content_h = HEADER_H
        + SECTION_GAP
        + 22.0
        + msg_height
        + file_count as f32 * FILE_ROW_HEIGHT
        + CONTENT_PAD;
    let max_panel_h = 700.0_f32;
    let panel_height = content_h.min(max_panel_h).max(HEADER_H + 60.0);
    let panel_height = panel_height.min(available_height);
    let width = panel_width(body_rect.width());

    let mut panel_rect = egui::Rect::from_min_size(
        egui::pos2(
            body_rect.right() - width - PANEL_MARGIN,
            body_rect.bottom() - panel_height - PANEL_MARGIN,
        ),
        egui::vec2(width, panel_height),
    );

    if bottom_offset > 0.0 {
        panel_rect = panel_rect.translate(egui::vec2(0.0, -bottom_offset));
    }

    let allowed_top = body_rect.top();
    if panel_rect.top() < allowed_top {
        let diff = allowed_top - panel_rect.top();
        panel_rect = panel_rect.translate(egui::vec2(0.0, diff));
    }

    render_panel_selected_commit(
        ui,
        panel_rect,
        state,
        commit_subject,
        commit_hash,
        commit_message,
        files,
    );
}

fn panel_width(body_width: f32) -> f32 {
    let available = (body_width - PANEL_MARGIN * 2.0).max(0.0);
    PANEL_WIDTH.min(available).max(0.0).min(available)
}

fn calc_panel_rect(body_rect: egui::Rect) -> egui::Rect {
    let available_height = (body_rect.height() - PANEL_MARGIN * 2.0).max(0.0);
    let height = PANEL_HEIGHT
        .min(available_height)
        .max(0.0)
        .min(available_height);
    let width = panel_width(body_rect.width());

    egui::Rect::from_min_size(
        egui::pos2(
            body_rect.right() - width - PANEL_MARGIN,
            body_rect.bottom() - height - PANEL_MARGIN,
        ),
        egui::vec2(width, height),
    )
}

fn render_panel(
    ui: &mut egui::Ui,
    panel_rect: egui::Rect,
    state: &mut State,
    header_text: &str,
    status: &Option<crate::git::models::RepoStatus>,
) {
    let orig_bottom = panel_rect.bottom();
    let panel_rect = if state.collapsed {
        let mut r = egui::Rect::from_min_size(
            panel_rect.left_top(),
            egui::vec2(panel_rect.width(), HEADER_H),
        );
        r = r.translate(egui::vec2(0.0, orig_bottom - r.bottom()));
        r
    } else {
        panel_rect
    };

    let fill = egui::Color32::from_rgb(36, 36, 36);
    let header_fill = egui::Color32::from_rgb(44, 44, 44);
    let stroke = egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(78, 78, 78));
    let muted = egui::Color32::from_rgb(172, 172, 172);

    if !state.collapsed {
        ui.painter().rect_filled(
            panel_rect.translate(egui::vec2(3.0, 3.0)),
            6,
            egui::Color32::from_black_alpha(80),
        );
    }
    ui.painter().rect_filled(panel_rect, 6, fill);
    ui.painter()
        .rect_stroke(panel_rect, 6, stroke, egui::StrokeKind::Inside);

    let header_rect = egui::Rect::from_min_size(
        panel_rect.left_top(),
        egui::vec2(panel_rect.width(), HEADER_H),
    );
    ui.painter().rect_filled(
        header_rect,
        egui::CornerRadius {
            nw: 6,
            ne: 6,
            sw: if state.collapsed { 6 } else { 0 },
            se: if state.collapsed { 6 } else { 0 },
        },
        header_fill,
    );

    if !state.collapsed {
        ui.painter().line_segment(
            [header_rect.left_bottom(), header_rect.right_bottom()],
            stroke,
        );
    }
    painter_text(
        ui,
        egui::pos2(header_rect.left() + 12.0, header_rect.center().y),
        GIT_COMMIT,
        15.0,
        ui.visuals().text_color(),
        egui::Align2::LEFT_CENTER,
    );
    painter_text(
        ui,
        egui::pos2(header_rect.left() + 34.0, header_rect.center().y),
        &format!("Commit to {}", header_text),
        12.0,
        ui.visuals().text_color(),
        egui::Align2::LEFT_CENTER,
    );
    if let Some(s) = status {
        header_stats(
            ui,
            header_rect,
            s.additions,
            s.deletions,
            s.files_changed,
            24.0,
        );
    }

    let toggle_icon = if state.collapsed {
        CARET_DOWN
    } else {
        CARET_UP
    };
    let toggle_rect = egui::Rect::from_center_size(
        egui::pos2(header_rect.right() - 14.0, header_rect.center().y),
        egui::vec2(20.0, 20.0),
    );
    let toggle_resp = ui.scope_builder(
        egui::UiBuilder::new()
            .id_salt("panel_toggle")
            .max_rect(toggle_rect),
        |ui| ui.button(egui::RichText::new(toggle_icon).size(11.0).color(muted)),
    );
    if toggle_resp.inner.clicked() {
        state.collapsed = !state.collapsed;
    }

    if state.collapsed {
        return;
    }

    let msg_box_rect = egui::Rect::from_min_max(
        egui::pos2(
            panel_rect.left() + CONTENT_PAD,
            panel_rect.bottom() - MSG_BOX_H - BTN_H - 8.0 - CONTENT_PAD,
        ),
        egui::pos2(
            panel_rect.right() - CONTENT_PAD,
            panel_rect.bottom() - BTN_H - 8.0 - CONTENT_PAD,
        ),
    );

    let content_left = panel_rect.left() + CONTENT_PAD;
    let content_right = panel_rect.right() - CONTENT_PAD;

    let options_h = options_height(state);
    let unstaged_bottom = msg_box_rect.top() - options_h - SECTION_GAP;
    let unstaged_top = header_rect.bottom() + SECTION_GAP;
    let unstaged_rect = egui::Rect::from_min_max(
        egui::pos2(content_left, unstaged_top),
        egui::pos2(content_right, unstaged_bottom),
    );

    ui.scope_builder(
        egui::UiBuilder::new()
            .id_salt("floating_commit_unstaged")
            .max_rect(unstaged_rect)
            .layout(egui::Layout::top_down(egui::Align::Min)),
        |ui| {
            if let Some(s) = status {
                top_strip(ui, s, muted, state);
            } else {
                top_strip_empty(ui, muted);
            }
            ui.add_space(4.0);
            if let Some(s) = status {
                unstaged_files_list(ui, &s.unstaged_files, muted, state);
            }
            ui.add_space(4.0);
            section_divider(ui);
            ui.add_space(4.0);
            if let Some(s) = status {
                staged_files_list(ui, &s.staged_files, muted, state);
            }
        },
    );

    let options_top = msg_box_rect.top() - options_h - SECTION_GAP;
    let options_rect = egui::Rect::from_min_max(
        egui::pos2(content_left, options_top),
        egui::pos2(content_right, msg_box_rect.top() - SECTION_GAP),
    );
    ui.scope_builder(
        egui::UiBuilder::new()
            .id_salt("floating_commit_options")
            .max_rect(options_rect)
            .layout(egui::Layout::top_down(egui::Align::Min)),
        |ui| {
            options_section(ui, state, muted);
        },
    );

    ui.scope_builder(
        egui::UiBuilder::new()
            .id_salt("floating_commit_msg")
            .max_rect(msg_box_rect),
        |ui| {
            message_box(ui, state);
        },
    );

    let btn_rect = egui::Rect::from_min_max(
        egui::pos2(panel_rect.left() + CONTENT_PAD, msg_box_rect.bottom() + 8.0),
        egui::pos2(
            panel_rect.right() - CONTENT_PAD,
            panel_rect.bottom() - CONTENT_PAD,
        ),
    );
    let has_staged = status.as_ref().is_some_and(|s| s.staged_count > 0);
    let has_unstaged = status.as_ref().is_some_and(|s| s.unstaged_count > 0);
    ui.scope_builder(
        egui::UiBuilder::new()
            .id_salt("floating_commit_btn")
            .max_rect(btn_rect),
        |ui| {
            let (color, icon, label) = if state.stash_mode {
                (
                    egui::Color32::from_rgb(138, 43, 226),
                    FOLDER,
                    "Stage changes to stash",
                )
            } else if has_unstaged {
                (
                    egui::Color32::from_rgb(39, 174, 96),
                    GIT_COMMIT,
                    "Stage & commit",
                )
            } else if has_staged {
                (egui::Color32::from_rgb(39, 174, 96), GIT_COMMIT, "Commit")
            } else {
                (
                    egui::Color32::from_rgb(80, 80, 80),
                    GIT_COMMIT,
                    "Nothing to commit",
                )
            };
            let enabled =
                (state.stash_mode || !state.title.is_empty()) && (has_staged || has_unstaged);
            ui.vertical_centered(|ui| {
                let btn = egui::Button::new(
                    egui::RichText::new(format!("{icon}  {label}"))
                        .size(12.0)
                        .color(egui::Color32::WHITE),
                )
                .fill(color)
                .corner_radius(6)
                .min_size(egui::vec2(ui.available_width(), ui.available_height()));
                if ui.add_enabled(enabled, btn).clicked() {
                    if state.stash_mode {
                        let message = if state.title.is_empty() {
                            None
                        } else if state.description.is_empty() {
                            Some(state.title.clone())
                        } else {
                            Some(format!("{}\n\n{}", state.title, state.description))
                        };
                        state.pending_stash_action = Some(StashAction::Save(message));
                    } else if has_unstaged {
                        state.queue_action(CommitAction::StageAll);
                        let message = if state.description.is_empty() {
                            state.title.clone()
                        } else {
                            format!("{}\n\n{}", state.title, state.description)
                        };
                        state.queue_action(CommitAction::Commit {
                            message,
                            amend: state.amend,
                            skip_hooks: state.skip_hooks,
                        });
                    } else {
                        let message = if state.description.is_empty() {
                            state.title.clone()
                        } else {
                            format!("{}\n\n{}", state.title, state.description)
                        };
                        state.queue_action(CommitAction::Commit {
                            message,
                            amend: state.amend,
                            skip_hooks: state.skip_hooks,
                        });
                    }
                }
            });
        },
    );

    if state.show_discard_confirm {
        show_discard_confirm(ui, panel_rect, state);
    }
}

fn render_panel_cached(
    ui: &mut egui::Ui,
    panel_rect: egui::Rect,
    state: &mut State,
    header_text: &str,
    app_state: &AppState,
) {
    let orig_bottom = panel_rect.bottom();
    let panel_rect = if state.collapsed {
        let mut r = egui::Rect::from_min_size(
            panel_rect.left_top(),
            egui::vec2(panel_rect.width(), HEADER_H),
        );
        r = r.translate(egui::vec2(0.0, orig_bottom - r.bottom()));
        r
    } else {
        panel_rect
    };

    let fill = egui::Color32::from_rgb(36, 36, 36);
    let header_fill = egui::Color32::from_rgb(44, 44, 44);
    let stroke = egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(78, 78, 78));
    let muted = egui::Color32::from_rgb(172, 172, 172);

    if !state.collapsed {
        ui.painter().rect_filled(
            panel_rect.translate(egui::vec2(3.0, 3.0)),
            6,
            egui::Color32::from_black_alpha(80),
        );
    }
    ui.painter().rect_filled(panel_rect, 6, fill);
    ui.painter()
        .rect_stroke(panel_rect, 6, stroke, egui::StrokeKind::Inside);

    let header_rect = egui::Rect::from_min_size(
        panel_rect.left_top(),
        egui::vec2(panel_rect.width(), HEADER_H),
    );
    ui.painter().rect_filled(
        header_rect,
        egui::CornerRadius {
            nw: 6,
            ne: 6,
            sw: if state.collapsed { 6 } else { 0 },
            se: if state.collapsed { 6 } else { 0 },
        },
        header_fill,
    );

    if !state.collapsed {
        ui.painter().line_segment(
            [header_rect.left_bottom(), header_rect.right_bottom()],
            stroke,
        );
    }
    painter_text(
        ui,
        egui::pos2(header_rect.left() + 12.0, header_rect.center().y),
        GIT_COMMIT,
        15.0,
        ui.visuals().text_color(),
        egui::Align2::LEFT_CENTER,
    );
    painter_text(
        ui,
        egui::pos2(header_rect.left() + 34.0, header_rect.center().y),
        &format!("Commit to {}", header_text),
        12.0,
        ui.visuals().text_color(),
        egui::Align2::LEFT_CENTER,
    );
    if let Some(s) = &app_state.cached_status {
        header_stats(
            ui,
            header_rect,
            s.additions,
            s.deletions,
            s.files_changed,
            24.0,
        );
    }

    let toggle_icon = if state.collapsed {
        CARET_DOWN
    } else {
        CARET_UP
    };
    let toggle_rect = egui::Rect::from_center_size(
        egui::pos2(header_rect.right() - 14.0, header_rect.center().y),
        egui::vec2(20.0, 20.0),
    );
    let toggle_resp = ui.scope_builder(
        egui::UiBuilder::new()
            .id_salt("panel_toggle")
            .max_rect(toggle_rect),
        |ui| ui.button(egui::RichText::new(toggle_icon).size(11.0).color(muted)),
    );
    if toggle_resp.inner.clicked() {
        state.collapsed = !state.collapsed;
    }

    if state.collapsed {
        return;
    }

    let msg_box_rect = egui::Rect::from_min_max(
        egui::pos2(
            panel_rect.left() + CONTENT_PAD,
            panel_rect.bottom() - MSG_BOX_H - BTN_H - 8.0 - CONTENT_PAD,
        ),
        egui::pos2(
            panel_rect.right() - CONTENT_PAD,
            panel_rect.bottom() - BTN_H - 8.0 - CONTENT_PAD,
        ),
    );

    let content_left = panel_rect.left() + CONTENT_PAD;
    let content_right = panel_rect.right() - CONTENT_PAD;

    let options_h = options_height(state);
    let unstaged_bottom = msg_box_rect.top() - options_h - SECTION_GAP;
    let unstaged_top = header_rect.bottom() + SECTION_GAP;
    let unstaged_rect = egui::Rect::from_min_max(
        egui::pos2(content_left, unstaged_top),
        egui::pos2(content_right, unstaged_bottom),
    );

    ui.scope_builder(
        egui::UiBuilder::new()
            .id_salt("floating_commit_unstaged")
            .max_rect(unstaged_rect)
            .layout(egui::Layout::top_down(egui::Align::Min)),
        |ui| {
            if let Some(s) = &app_state.cached_status {
                top_strip_cached(ui, s, muted, state);
            } else {
                top_strip_empty(ui, muted);
            }
            ui.add_space(4.0);
            if let Some(s) = &app_state.cached_status {
                unstaged_files_list_cached(ui, &s.unstaged_files, muted, state);
            }
            ui.add_space(4.0);
            section_divider(ui);
            ui.add_space(4.0);
            if let Some(s) = &app_state.cached_status {
                staged_files_list_cached(ui, &s.staged_files, muted, state);
            }
        },
    );

    let options_top = msg_box_rect.top() - options_h - SECTION_GAP;
    let options_rect = egui::Rect::from_min_max(
        egui::pos2(content_left, options_top),
        egui::pos2(content_right, msg_box_rect.top() - SECTION_GAP),
    );
    ui.scope_builder(
        egui::UiBuilder::new()
            .id_salt("floating_commit_options")
            .max_rect(options_rect)
            .layout(egui::Layout::top_down(egui::Align::Min)),
        |ui| {
            options_section(ui, state, muted);
        },
    );

    ui.scope_builder(
        egui::UiBuilder::new()
            .id_salt("floating_commit_msg")
            .max_rect(msg_box_rect),
        |ui| {
            message_box(ui, state);
        },
    );

    let btn_rect = egui::Rect::from_min_max(
        egui::pos2(panel_rect.left() + CONTENT_PAD, msg_box_rect.bottom() + 8.0),
        egui::pos2(
            panel_rect.right() - CONTENT_PAD,
            panel_rect.bottom() - CONTENT_PAD,
        ),
    );
    let cached_staged = app_state
        .cached_status
        .as_ref()
        .map_or(0, |s| s.staged_count);
    let cached_unstaged = app_state
        .cached_status
        .as_ref()
        .map_or(0, |s| s.unstaged_count);
    let has_staged = cached_staged > 0;
    let has_unstaged = cached_unstaged > 0;
    ui.scope_builder(
        egui::UiBuilder::new()
            .id_salt("floating_commit_btn")
            .max_rect(btn_rect),
        |ui| {
            let (color, icon, label) = if state.stash_mode {
                (
                    egui::Color32::from_rgb(138, 43, 226),
                    FOLDER,
                    "Stage changes to stash",
                )
            } else if has_unstaged {
                (
                    egui::Color32::from_rgb(39, 174, 96),
                    GIT_COMMIT,
                    "Stage & commit",
                )
            } else if has_staged {
                (egui::Color32::from_rgb(39, 174, 96), GIT_COMMIT, "Commit")
            } else {
                (
                    egui::Color32::from_rgb(80, 80, 80),
                    GIT_COMMIT,
                    "Nothing to commit",
                )
            };
            let enabled =
                (state.stash_mode || !state.title.is_empty()) && (has_staged || has_unstaged);
            ui.vertical_centered(|ui| {
                let btn = egui::Button::new(
                    egui::RichText::new(format!("{icon}  {label}"))
                        .size(12.0)
                        .color(egui::Color32::WHITE),
                )
                .fill(color)
                .corner_radius(6)
                .min_size(egui::vec2(ui.available_width(), ui.available_height()));
                if ui.add_enabled(enabled, btn).clicked() {
                    if state.stash_mode {
                        let message = if state.title.is_empty() {
                            None
                        } else if state.description.is_empty() {
                            Some(state.title.clone())
                        } else {
                            Some(format!("{}\n\n{}", state.title, state.description))
                        };
                        state.pending_stash_action = Some(StashAction::Save(message));
                    } else if has_unstaged {
                        state.queue_action(CommitAction::StageAll);
                        let message = if state.description.is_empty() {
                            state.title.clone()
                        } else {
                            format!("{}\n\n{}", state.title, state.description)
                        };
                        state.queue_action(CommitAction::Commit {
                            message,
                            amend: state.amend,
                            skip_hooks: state.skip_hooks,
                        });
                    } else {
                        let message = if state.description.is_empty() {
                            state.title.clone()
                        } else {
                            format!("{}\n\n{}", state.title, state.description)
                        };
                        state.queue_action(CommitAction::Commit {
                            message,
                            amend: state.amend,
                            skip_hooks: state.skip_hooks,
                        });
                    }
                }
            });
        },
    );

    if state.show_discard_confirm {
        show_discard_confirm(ui, panel_rect, state);
    }
}

#[allow(clippy::too_many_arguments)]
fn render_panel_selected_commit(
    ui: &mut egui::Ui,
    panel_rect: egui::Rect,
    state: &mut State,
    commit_subject: &str,
    commit_hash: &str,
    commit_message: &str,
    files: &[crate::git::models::FileStatus],
) {
    let orig_bottom = panel_rect.bottom();
    let panel_rect = if state.collapsed {
        let mut r = egui::Rect::from_min_size(
            panel_rect.left_top(),
            egui::vec2(panel_rect.width(), HEADER_H),
        );
        r = r.translate(egui::vec2(0.0, orig_bottom - r.bottom()));
        r
    } else {
        panel_rect
    };

    let fill = egui::Color32::from_rgb(36, 36, 36);
    let header_fill = egui::Color32::from_rgb(44, 44, 44);
    let stroke = egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(78, 78, 78));
    let muted = egui::Color32::from_rgb(172, 172, 172);

    if !state.collapsed {
        ui.painter().rect_filled(
            panel_rect.translate(egui::vec2(3.0, 3.0)),
            6,
            egui::Color32::from_black_alpha(80),
        );
    }
    ui.painter().rect_filled(panel_rect, 6, fill);
    ui.painter()
        .rect_stroke(panel_rect, 6, stroke, egui::StrokeKind::Inside);

    let header_rect = egui::Rect::from_min_size(
        panel_rect.left_top(),
        egui::vec2(panel_rect.width(), HEADER_H),
    );
    ui.painter().rect_filled(
        header_rect,
        egui::CornerRadius {
            nw: 6,
            ne: 6,
            sw: if state.collapsed { 6 } else { 0 },
            se: if state.collapsed { 6 } else { 0 },
        },
        header_fill,
    );

    if !state.collapsed {
        ui.painter().line_segment(
            [header_rect.left_bottom(), header_rect.right_bottom()],
            stroke,
        );
    }

    painter_text(
        ui,
        egui::pos2(header_rect.left() + 12.0, header_rect.center().y),
        GIT_COMMIT,
        15.0,
        ui.visuals().text_color(),
        egui::Align2::LEFT_CENTER,
    );

    let short = if commit_hash.len() > 7 {
        &commit_hash[..7]
    } else {
        commit_hash
    };
    let header_text = if commit_subject.is_empty() {
        format!("Commit {}", short)
    } else {
        format!("{} {}", short, commit_subject)
    };
    let max_header_chars = ((panel_rect.width() - 130.0) / (11.0 * 0.55)) as usize;
    let header_display = truncate_str(&header_text, max_header_chars.max(10));

    painter_text(
        ui,
        egui::pos2(header_rect.left() + 34.0, header_rect.center().y),
        &header_display,
        11.0,
        ui.visuals().text_color(),
        egui::Align2::LEFT_CENTER,
    );

    let total_additions: usize = files.iter().map(|f| f.additions).sum();
    let total_deletions: usize = files.iter().map(|f| f.deletions).sum();
    header_stats(
        ui,
        header_rect,
        total_additions,
        total_deletions,
        files.len(),
        24.0,
    );

    let toggle_icon = if state.collapsed {
        CARET_DOWN
    } else {
        CARET_UP
    };
    let toggle_rect = egui::Rect::from_center_size(
        egui::pos2(header_rect.right() - 14.0, header_rect.center().y),
        egui::vec2(20.0, 20.0),
    );
    let toggle_resp = ui.scope_builder(
        egui::UiBuilder::new()
            .id_salt("panel_toggle")
            .max_rect(toggle_rect),
        |ui| ui.button(egui::RichText::new(toggle_icon).size(11.0).color(muted)),
    );
    if toggle_resp.inner.clicked() {
        state.collapsed = !state.collapsed;
    }

    if state.collapsed {
        return;
    }

    let content_left = panel_rect.left() + CONTENT_PAD;
    let content_right = panel_rect.right() - CONTENT_PAD;

    let files_rect = egui::Rect::from_min_max(
        egui::pos2(content_left, header_rect.bottom() + SECTION_GAP),
        egui::pos2(content_right, panel_rect.bottom() - CONTENT_PAD),
    );

    ui.scope_builder(
        egui::UiBuilder::new()
            .id_salt("selected_commit_files")
            .max_rect(files_rect)
            .layout(egui::Layout::top_down(egui::Align::Min)),
        |ui| {
            if files.is_empty() {
                ui.add_space(6.0);
                ui.label(
                    egui::RichText::new("No files changed")
                        .size(10.0)
                        .color(muted),
                );
            } else {
                section_header(ui, "Files changed", files.len(), muted);
                ui.add_space(4.0);
                egui::ScrollArea::vertical()
                    .id_salt("selected_commit_files_scroll")
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        if !commit_message.is_empty() {
                            let msg_bg = egui::Color32::from_rgb(40, 40, 40);
                            let msg_stroke =
                                egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(60, 60, 60));
                            let avail_w = ui.available_width();
                            let galley = ui.painter().layout(
                                commit_message.to_string(),
                                egui::FontId::proportional(11.0),
                                egui::Color32::from_rgb(180, 180, 180),
                                avail_w - 16.0,
                            );
                            let full_h = galley.size().y + 16.0;
                            let collapsed_h = full_h.min(120.0);
                            let msg_h = if state.msg_expanded {
                                full_h
                            } else {
                                collapsed_h
                            };
                            let (msg_rect, response) = ui.allocate_exact_size(
                                egui::vec2(avail_w, msg_h),
                                egui::Sense::click(),
                            );
                            if response.clicked() {
                                state.msg_expanded = !state.msg_expanded;
                            }
                            if response.hovered() {
                                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                            }
                            ui.painter().rect_filled(msg_rect, 4.0, msg_bg);
                            ui.painter().rect_stroke(
                                msg_rect,
                                4.0,
                                msg_stroke,
                                egui::StrokeKind::Inside,
                            );
                            let clip = msg_rect.shrink2(egui::vec2(4.0, 4.0));
                            let clipped = ui.painter().with_clip_rect(clip);
                            clipped.galley(
                                msg_rect.min + egui::vec2(8.0, 8.0),
                                galley,
                                egui::Color32::from_rgb(180, 180, 180),
                            );
                            ui.add_space(8.0);
                        }
                        for file in files {
                            file_row_view_only(ui, file);
                        }
                    });
            }
        },
    );
}

fn file_row_view_only(ui: &mut egui::Ui, file: &crate::git::models::FileStatus) {
    let (_, icon_color) = file_icon_for_kind(&file.kind);
    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), FILE_ROW_HEIGHT),
        egui::Sense::hover(),
    );

    if response.hovered() {
        ui.painter()
            .rect_filled(rect, 3.0, egui::Color32::from_rgb(48, 48, 48));
    }

    let icon_x = rect.left() + 4.0;
    let icon_rect = egui::Rect::from_center_size(
        egui::pos2(rect.left() + 10.0, rect.center().y),
        egui::vec2(13.0, 13.0),
    );
    crate::ui::core::filetree::paint_file_icon_rect(ui, icon_rect, &file.path, icon_color);

    let path_x = icon_x + 16.0;
    let path_width = rect.width() - 80.0 - (path_x - rect.left());
    let display = truncate_path(&file.path, path_width, 10.0);
    painter_text(
        ui,
        egui::pos2(path_x, rect.center().y),
        &display,
        10.0,
        ui.visuals().text_color(),
        egui::Align2::LEFT_CENTER,
    );

    let stats_x = rect.right() - 72.0;
    if file.additions > 0 || file.deletions > 0 {
        painter_text(
            ui,
            egui::pos2(stats_x, rect.center().y),
            &format!("+{}", file.additions),
            9.0,
            egui::Color32::from_rgb(78, 190, 116),
            egui::Align2::LEFT_CENTER,
        );
        painter_text(
            ui,
            egui::pos2(stats_x + 32.0, rect.center().y),
            &format!("-{}", file.deletions),
            9.0,
            egui::Color32::from_rgb(230, 92, 92),
            egui::Align2::LEFT_CENTER,
        );
    }
}

fn truncate_str(s: &str, max_chars: usize) -> String {
    let char_count = s.chars().count();
    if char_count <= max_chars {
        return s.to_string();
    }
    let truncated: String = s.chars().take(max_chars.saturating_sub(3)).collect();
    format!("{}...", truncated)
}

fn top_strip(
    ui: &mut egui::Ui,
    status: &crate::git::models::RepoStatus,
    muted: egui::Color32,
    state: &mut State,
) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing = egui::vec2(6.0, 0.0);

        let red = egui::Color32::from_rgb(231, 76, 60);
        let discard_btn = egui::Button::new(
            egui::RichText::new(TRASH.to_string())
                .size(9.0)
                .color(egui::Color32::WHITE),
        )
        .fill(red)
        .min_size(egui::vec2(0.0, 18.0));
        if ui.add(discard_btn).clicked() {
            state.show_discard_confirm = true;
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            icon_label(
                ui,
                WARNING,
                &format!("{}", status.unstaged_count),
                "Unstaged",
                muted,
            );
            separator(ui);
            icon_label(
                ui,
                LIST_CHECKS,
                &format!("{}", status.staged_count),
                "Staged",
                muted,
            );
            separator(ui);
            icon_label(ui, GIT_BRANCH, &status.branch, "Current branch", muted);
        });
    });
}

fn top_strip_cached(
    ui: &mut egui::Ui,
    status: &crate::state::CachedRepoStatus,
    muted: egui::Color32,
    state: &mut State,
) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing = egui::vec2(6.0, 0.0);

        let red = egui::Color32::from_rgb(231, 76, 60);
        let discard_btn = egui::Button::new(
            egui::RichText::new(TRASH.to_string())
                .size(9.0)
                .color(egui::Color32::WHITE),
        )
        .fill(red)
        .min_size(egui::vec2(0.0, 18.0));
        if ui.add(discard_btn).clicked() {
            state.show_discard_confirm = true;
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            icon_label(
                ui,
                WARNING,
                &format!("{}", status.unstaged_count),
                "Unstaged",
                muted,
            );
            separator(ui);
            icon_label(
                ui,
                LIST_CHECKS,
                &format!("{}", status.staged_count),
                "Staged",
                muted,
            );
            separator(ui);
            icon_label(ui, GIT_BRANCH, &status.branch, "Current branch", muted);
        });
    });
}

fn top_strip_empty(ui: &mut egui::Ui, muted: egui::Color32) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing = egui::vec2(7.0, 0.0);
        icon_label(ui, GIT_BRANCH, "no repo", "No repository open", muted);
    });
}

fn header_stats(
    ui: &egui::Ui,
    header_rect: egui::Rect,
    additions: usize,
    deletions: usize,
    files: usize,
    right_margin: f32,
) {
    let y = header_rect.center().y;
    painter_text(
        ui,
        egui::pos2(header_rect.right() - 110.0 - right_margin, y),
        &format!("+{}", additions),
        11.0,
        egui::Color32::from_rgb(78, 190, 116),
        egui::Align2::LEFT_CENTER,
    );
    painter_text(
        ui,
        egui::pos2(header_rect.right() - 74.0 - right_margin, y),
        &format!("-{}", deletions),
        11.0,
        egui::Color32::from_rgb(230, 92, 92),
        egui::Align2::LEFT_CENTER,
    );
    painter_text(
        ui,
        egui::pos2(header_rect.right() - 40.0 - right_margin, y),
        &format!("{}", files),
        11.0,
        egui::Color32::from_rgb(172, 172, 172),
        egui::Align2::LEFT_CENTER,
    );
}

fn message_box(ui: &mut egui::Ui, state: &mut State) {
    let section_rect = ui.max_rect();
    let section_fill = egui::Color32::from_rgb(40, 40, 40);
    let section_stroke = egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(72, 72, 72));
    let editor_fill = egui::Color32::from_rgb(49, 49, 49);

    ui.painter().rect_filled(section_rect, 6, section_fill);
    ui.painter()
        .rect_stroke(section_rect, 6, section_stroke, egui::StrokeKind::Inside);

    let inner_rect = section_rect.shrink2(egui::vec2(6.0, 6.0));
    ui.scope_builder(
        egui::UiBuilder::new()
            .max_rect(inner_rect)
            .layout(egui::Layout::top_down(egui::Align::Min)),
        |ui| {
            ui.spacing_mut().item_spacing = egui::vec2(4.0, 4.0);

            let title_len = state.title.chars().count();
            let remaining = MAX_TITLE_LEN as i64 - title_len as i64;

            let title_color = if remaining < 0 {
                egui::Color32::from_rgb(230, 92, 92)
            } else if remaining <= 10 {
                egui::Color32::from_rgb(252, 197, 34)
            } else {
                egui::Color32::from_rgb(140, 140, 140)
            };

            ui.horizontal(|ui| {
                let title_edit = egui::TextEdit::singleline(&mut state.title)
                    .hint_text("Commit title")
                    .frame(egui::Frame::NONE)
                    .background_color(editor_fill)
                    .desired_width((ui.available_width() - 40.0).max(0.0));
                ui.add(title_edit);

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        egui::RichText::new(format!("{}", remaining))
                            .size(9.0)
                            .color(title_color),
                    );
                });
            });

            let desc_edit = egui::TextEdit::multiline(&mut state.description)
                .hint_text("Description (optional)")
                .frame(egui::Frame::NONE)
                .background_color(editor_fill)
                .desired_rows(2);
            ui.add_sized([ui.available_width(), 36.0], desc_edit);
        },
    );
}

fn unstaged_files_list(
    ui: &mut egui::Ui,
    files: &[crate::git::models::FileStatus],
    muted: egui::Color32,
    state: &mut State,
) {
    if files.is_empty() {
        ui.label(
            egui::RichText::new("No unstaged changes")
                .size(10.0)
                .color(muted),
        );
        return;
    }

    let list_height = (files.len().min(7) as f32 * FILE_ROW_HEIGHT).max(FILE_ROW_HEIGHT * 5.0);
    ui.allocate_ui(egui::vec2(ui.available_width(), list_height), |ui| {
        egui::ScrollArea::vertical()
            .id_salt("unstaged_files")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                unstaged_section_header(ui, files.len(), muted, state);
                for file in files {
                    file_row_unstaged(ui, file, muted, state);
                }
            });
    });
}

fn unstaged_files_list_cached(
    ui: &mut egui::Ui,
    files: &[CachedFileStatus],
    muted: egui::Color32,
    state: &mut State,
) {
    if files.is_empty() {
        ui.label(
            egui::RichText::new("No unstaged changes")
                .size(10.0)
                .color(muted),
        );
        return;
    }

    let list_height = (files.len().min(7) as f32 * FILE_ROW_HEIGHT).max(FILE_ROW_HEIGHT * 5.0);
    ui.allocate_ui(egui::vec2(ui.available_width(), list_height), |ui| {
        egui::ScrollArea::vertical()
            .id_salt("unstaged_files")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                unstaged_section_header(ui, files.len(), muted, state);
                for file in files {
                    file_row_unstaged_cached(ui, file, muted, state);
                }
            });
    });
}

fn staged_files_list(
    ui: &mut egui::Ui,
    files: &[crate::git::models::FileStatus],
    muted: egui::Color32,
    state: &mut State,
) {
    if files.is_empty() {
        return;
    }

    let list_height = (files.len().min(7) as f32 * FILE_ROW_HEIGHT).max(FILE_ROW_HEIGHT * 5.0);
    ui.allocate_ui(egui::vec2(ui.available_width(), list_height), |ui| {
        egui::ScrollArea::vertical()
            .id_salt("staged_files")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                staged_section_header(ui, files.len(), muted, state);
                for file in files {
                    file_row_staged(ui, file, muted, state);
                }
            });
    });
}

fn staged_files_list_cached(
    ui: &mut egui::Ui,
    files: &[CachedFileStatus],
    muted: egui::Color32,
    state: &mut State,
) {
    if files.is_empty() {
        return;
    }

    let list_height = (files.len().min(7) as f32 * FILE_ROW_HEIGHT).max(FILE_ROW_HEIGHT * 5.0);
    ui.allocate_ui(egui::vec2(ui.available_width(), list_height), |ui| {
        egui::ScrollArea::vertical()
            .id_salt("staged_files")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                staged_section_header(ui, files.len(), muted, state);
                for file in files {
                    file_row_staged_cached(ui, file, muted, state);
                }
            });
    });
}

fn section_header(ui: &mut egui::Ui, label: &str, count: usize, muted: egui::Color32) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing = egui::vec2(4.0, 0.0);
        ui.label(egui::RichText::new(label).size(9.0).color(muted).strong());
        ui.label(
            egui::RichText::new(format!("({})", count))
                .size(9.0)
                .color(egui::Color32::from_rgb(120, 120, 120)),
        );
    });
}

fn unstaged_section_header(
    ui: &mut egui::Ui,
    count: usize,
    muted: egui::Color32,
    state: &mut State,
) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing = egui::vec2(4.0, 0.0);
        ui.label(
            egui::RichText::new("Unstaged")
                .size(9.0)
                .color(muted)
                .strong(),
        );
        ui.label(
            egui::RichText::new(format!("({})", count))
                .size(9.0)
                .color(egui::Color32::from_rgb(120, 120, 120)),
        );

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let green = egui::Color32::from_rgb(39, 174, 96);
            let stage_all = egui::Button::new(
                egui::RichText::new(format!("{PLUS} Stage all"))
                    .size(9.0)
                    .color(egui::Color32::WHITE),
            )
            .fill(green)
            .min_size(egui::vec2(0.0, 18.0));
            if ui.add(stage_all).clicked() {
                state.queue_action(CommitAction::StageAll);
            }
        });
    });
}

fn staged_section_header(ui: &mut egui::Ui, count: usize, muted: egui::Color32, state: &mut State) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing = egui::vec2(4.0, 0.0);
        ui.label(
            egui::RichText::new("Staged")
                .size(9.0)
                .color(muted)
                .strong(),
        );
        ui.label(
            egui::RichText::new(format!("({})", count))
                .size(9.0)
                .color(egui::Color32::from_rgb(120, 120, 120)),
        );

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let red = egui::Color32::from_rgb(231, 76, 60);
            let unstage_all = egui::Button::new(
                egui::RichText::new(format!("{MINUS} Unstage all"))
                    .size(9.0)
                    .color(egui::Color32::WHITE),
            )
            .fill(red)
            .min_size(egui::vec2(0.0, 18.0));
            if ui.add(unstage_all).clicked() {
                state.queue_action(CommitAction::UnstageAll);
            }
        });
    });
}

const OPTIONS_H_COLLAPSED: f32 = 20.0;
const OPTIONS_H_EXPANDED: f32 = 40.0;

fn options_section(ui: &mut egui::Ui, state: &mut State, muted: egui::Color32) {
    let (toggle_rect, toggle_response) =
        ui.allocate_exact_size(egui::vec2(ui.available_width(), 16.0), egui::Sense::click());

    if toggle_response.clicked() {
        state.options_expanded = !state.options_expanded;
    }

    let arrow = if state.options_expanded {
        CARET_DOWN
    } else {
        CARET_RIGHT
    };
    let hovered = toggle_rect.contains(
        ui.input(|i| i.pointer.hover_pos())
            .unwrap_or(egui::Pos2::ZERO),
    );
    let text_color = if hovered {
        ui.visuals().text_color()
    } else {
        muted
    };
    painter_text(
        ui,
        egui::pos2(toggle_rect.left() + 2.0, toggle_rect.center().y),
        &format!("{} Options", arrow),
        10.0,
        text_color,
        egui::Align2::LEFT_CENTER,
    );

    let (mode_label, mode_icon, mode_color) = if state.stash_mode {
        ("Commit", GIT_COMMIT, egui::Color32::from_rgb(39, 174, 96))
    } else {
        ("Stash", FOLDER, egui::Color32::from_rgb(138, 43, 226))
    };
    let font_id = egui::FontId::proportional(9.0);
    let mode_marker = format!("{mode_icon} {mode_label}");
    let mode_text_width = ui
        .painter()
        .layout_no_wrap(mode_marker.clone(), font_id.clone(), egui::Color32::WHITE)
        .rect
        .width();
    let mode_rect = egui::Rect::from_min_size(
        egui::pos2(
            toggle_rect.right() - mode_text_width - 16.0,
            toggle_rect.center().y - 7.0,
        ),
        egui::vec2(mode_text_width + 12.0, 14.0),
    );
    let mode_resp = ui.interact(
        mode_rect,
        ui.make_persistent_id("commit_stash_toggle"),
        egui::Sense::click(),
    );
    if mode_resp.clicked() {
        state.stash_mode = !state.stash_mode;
    }
    ui.painter()
        .rect_filled(mode_rect, 3.0, mode_color.linear_multiply(0.18));
    ui.painter().rect_stroke(
        mode_rect,
        3.0,
        egui::Stroke::new(1.0_f32, mode_color),
        egui::StrokeKind::Inside,
    );
    painter_text(
        ui,
        mode_rect.center(),
        &mode_marker,
        9.0,
        mode_color,
        egui::Align2::CENTER_CENTER,
    );

    if state.options_expanded {
        ui.add_space(2.0);
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = egui::vec2(8.0, 0.0);
            ui.checkbox(&mut state.amend, egui::RichText::new("Amend").size(10.0));
            ui.checkbox(
                &mut state.sign_off,
                egui::RichText::new("Sign-off").size(10.0),
            );
            ui.checkbox(
                &mut state.skip_hooks,
                egui::RichText::new("Skip hooks").size(10.0),
            );
        });
    }
}

pub fn options_height(state: &State) -> f32 {
    if state.options_expanded {
        OPTIONS_H_EXPANDED
    } else {
        OPTIONS_H_COLLAPSED
    }
}

fn file_row_unstaged(
    ui: &mut egui::Ui,
    file: &crate::git::models::FileStatus,
    _muted: egui::Color32,
    state: &mut State,
) {
    use crate::git::models::FileChangeKind;
    let (_, icon_color) = file_icon_for_kind(&file.kind);
    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), FILE_ROW_HEIGHT),
        egui::Sense::click(),
    );

    let hovered = response.hovered();

    if hovered && response.clicked() {
        state.queue_action(CommitAction::StageFile(file.path.clone()));
    }
    if hovered {
        ui.painter()
            .rect_filled(rect, 3.0, egui::Color32::from_rgb(48, 48, 48));
    }

    let icon_x = rect.left() + 4.0;
    let icon_rect = egui::Rect::from_center_size(
        egui::pos2(rect.left() + 10.0, rect.center().y),
        egui::vec2(13.0, 13.0),
    );
    crate::ui::core::filetree::paint_file_icon_rect(ui, icon_rect, &file.path, icon_color);

    let path_x = icon_x + 16.0;
    let renamed = matches!(file.kind, FileChangeKind::Renamed);
    let arrow_space = if renamed { 12.0 } else { 0.0 };
    let stats_reserved = 80.0;
    let path_width = rect.width() - stats_reserved - arrow_space - (path_x - rect.left());

    if renamed {
        if let Some(ref old) = file.old_path {
            let old_display = truncate_path(old, path_width * 0.4, 10.0);
            let new_display = truncate_path(&file.path, path_width * 0.4, 10.0);
            painter_text(
                ui,
                egui::pos2(path_x, rect.center().y),
                &old_display,
                10.0,
                egui::Color32::from_rgb(150, 150, 150),
                egui::Align2::LEFT_CENTER,
            );
            let old_galley = ui.painter().layout_no_wrap(
                old_display,
                egui::FontId::proportional(10.0),
                egui::Color32::from_rgb(150, 150, 150),
            );
            let arrow_x = path_x + old_galley.size().x + 2.0;
            painter_text(
                ui,
                egui::pos2(arrow_x, rect.center().y),
                "→",
                10.0,
                egui::Color32::from_rgb(151, 113, 255),
                egui::Align2::LEFT_CENTER,
            );
            let arrow_galley = ui.painter().layout_no_wrap(
                "→".to_string(),
                egui::FontId::proportional(10.0),
                egui::Color32::from_rgb(151, 113, 255),
            );
            let new_x = arrow_x + arrow_galley.size().x + 2.0;
            painter_text(
                ui,
                egui::pos2(new_x, rect.center().y),
                &new_display,
                10.0,
                ui.visuals().text_color(),
                egui::Align2::LEFT_CENTER,
            );
        } else {
            let display = truncate_path(&file.path, path_width, 10.0);
            painter_text(
                ui,
                egui::pos2(path_x, rect.center().y),
                &display,
                10.0,
                ui.visuals().text_color(),
                egui::Align2::LEFT_CENTER,
            );
        }
    } else {
        let display = truncate_path(&file.path, path_width, 10.0);
        painter_text(
            ui,
            egui::pos2(path_x, rect.center().y),
            &display,
            10.0,
            ui.visuals().text_color(),
            egui::Align2::LEFT_CENTER,
        );
    }

    let stats_x = rect.right() - 72.0;
    if file.additions > 0 || file.deletions > 0 {
        painter_text(
            ui,
            egui::pos2(stats_x, rect.center().y),
            &format!("+{}", file.additions),
            9.0,
            egui::Color32::from_rgb(78, 190, 116),
            egui::Align2::LEFT_CENTER,
        );
        painter_text(
            ui,
            egui::pos2(stats_x + 32.0, rect.center().y),
            &format!("-{}", file.deletions),
            9.0,
            egui::Color32::from_rgb(230, 92, 92),
            egui::Align2::LEFT_CENTER,
        );
    }

    if hovered {
        let btn_rect = egui::Rect::from_center_size(
            egui::pos2(rect.right() - 14.0, rect.center().y),
            egui::vec2(18.0, 18.0),
        );
        ui.painter()
            .rect_filled(btn_rect, 4.0, egui::Color32::from_rgb(58, 58, 58));
        ui.painter().text(
            btn_rect.center(),
            egui::Align2::CENTER_CENTER,
            PLUS,
            egui::FontId::proportional(10.0),
            egui::Color32::WHITE,
        );
        if ui
            .interact(
                btn_rect,
                ui.make_persistent_id(format!("staged_btn_{}", file.path)),
                egui::Sense::click(),
            )
            .clicked()
        {
            state.queue_action(CommitAction::StageFile(file.path.clone()));
        }
    }
}

fn file_row_unstaged_cached(
    ui: &mut egui::Ui,
    file: &CachedFileStatus,
    _muted: egui::Color32,
    state: &mut State,
) {
    let (_, icon_color) = cached_file_icon_for_kind(&file.kind);
    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), FILE_ROW_HEIGHT),
        egui::Sense::click(),
    );

    let hovered = response.hovered();

    if hovered && response.clicked() {
        state.queue_action(CommitAction::StageFile(file.path.clone()));
    }
    if hovered {
        ui.painter()
            .rect_filled(rect, 3.0, egui::Color32::from_rgb(48, 48, 48));
    }

    let icon_x = rect.left() + 4.0;
    let icon_rect = egui::Rect::from_center_size(
        egui::pos2(rect.left() + 10.0, rect.center().y),
        egui::vec2(13.0, 13.0),
    );
    crate::ui::core::filetree::paint_file_icon_rect(ui, icon_rect, &file.path, icon_color);

    let path_x = icon_x + 16.0;
    let path_width = rect.width() - 80.0 - (path_x - rect.left());
    let display = truncate_path(&file.path, path_width, 10.0);
    painter_text(
        ui,
        egui::pos2(path_x, rect.center().y),
        &display,
        10.0,
        ui.visuals().text_color(),
        egui::Align2::LEFT_CENTER,
    );

    let stats_x = rect.right() - 72.0;
    if file.additions > 0 || file.deletions > 0 {
        painter_text(
            ui,
            egui::pos2(stats_x, rect.center().y),
            &format!("+{}", file.additions),
            9.0,
            egui::Color32::from_rgb(78, 190, 116),
            egui::Align2::LEFT_CENTER,
        );
        painter_text(
            ui,
            egui::pos2(stats_x + 32.0, rect.center().y),
            &format!("-{}", file.deletions),
            9.0,
            egui::Color32::from_rgb(230, 92, 92),
            egui::Align2::LEFT_CENTER,
        );
    }

    if hovered {
        let btn_rect = egui::Rect::from_center_size(
            egui::pos2(rect.right() - 14.0, rect.center().y),
            egui::vec2(18.0, 18.0),
        );
        ui.painter()
            .rect_filled(btn_rect, 4.0, egui::Color32::from_rgb(58, 58, 58));
        ui.painter().text(
            btn_rect.center(),
            egui::Align2::CENTER_CENTER,
            PLUS,
            egui::FontId::proportional(10.0),
            egui::Color32::WHITE,
        );
        if ui
            .interact(
                btn_rect,
                ui.make_persistent_id(format!("staged_cached_btn_{}", file.path)),
                egui::Sense::click(),
            )
            .clicked()
        {
            state.queue_action(CommitAction::StageFile(file.path.clone()));
        }
    }
}

fn file_row_staged(
    ui: &mut egui::Ui,
    file: &crate::git::models::FileStatus,
    _muted: egui::Color32,
    state: &mut State,
) {
    use crate::git::models::FileChangeKind;
    let (_, icon_color) = file_icon_for_kind(&file.kind);
    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), FILE_ROW_HEIGHT),
        egui::Sense::click(),
    );

    let hovered = response.hovered();

    if hovered && response.clicked() {
        state.queue_action(CommitAction::UnstageFile(file.path.clone()));
    }
    if hovered {
        ui.painter()
            .rect_filled(rect, 3.0, egui::Color32::from_rgb(48, 48, 48));
    }

    let icon_x = rect.left() + 4.0;
    let icon_rect = egui::Rect::from_center_size(
        egui::pos2(rect.left() + 10.0, rect.center().y),
        egui::vec2(13.0, 13.0),
    );
    crate::ui::core::filetree::paint_file_icon_rect(ui, icon_rect, &file.path, icon_color);

    let path_x = icon_x + 16.0;
    let renamed = matches!(file.kind, FileChangeKind::Renamed);
    let stats_reserved = 80.0;
    let path_width = rect.width() - stats_reserved - (path_x - rect.left());

    if renamed {
        if let Some(ref old) = file.old_path {
            let old_display = truncate_path(old, path_width * 0.4, 10.0);
            let new_display = truncate_path(&file.path, path_width * 0.4, 10.0);
            painter_text(
                ui,
                egui::pos2(path_x, rect.center().y),
                &old_display,
                10.0,
                egui::Color32::from_rgb(150, 150, 150),
                egui::Align2::LEFT_CENTER,
            );
            let old_galley = ui.painter().layout_no_wrap(
                old_display,
                egui::FontId::proportional(10.0),
                egui::Color32::from_rgb(150, 150, 150),
            );
            let arrow_x = path_x + old_galley.size().x + 2.0;
            painter_text(
                ui,
                egui::pos2(arrow_x, rect.center().y),
                "→",
                10.0,
                egui::Color32::from_rgb(151, 113, 255),
                egui::Align2::LEFT_CENTER,
            );
            let arrow_galley = ui.painter().layout_no_wrap(
                "→".to_string(),
                egui::FontId::proportional(10.0),
                egui::Color32::from_rgb(151, 113, 255),
            );
            let new_x = arrow_x + arrow_galley.size().x + 2.0;
            painter_text(
                ui,
                egui::pos2(new_x, rect.center().y),
                &new_display,
                10.0,
                ui.visuals().text_color(),
                egui::Align2::LEFT_CENTER,
            );
        } else {
            let display = truncate_path(&file.path, path_width, 10.0);
            painter_text(
                ui,
                egui::pos2(path_x, rect.center().y),
                &display,
                10.0,
                ui.visuals().text_color(),
                egui::Align2::LEFT_CENTER,
            );
        }
    } else {
        let display = truncate_path(&file.path, path_width, 10.0);
        painter_text(
            ui,
            egui::pos2(path_x, rect.center().y),
            &display,
            10.0,
            ui.visuals().text_color(),
            egui::Align2::LEFT_CENTER,
        );
    }

    let stats_x = rect.right() - 72.0;
    if file.additions > 0 || file.deletions > 0 {
        painter_text(
            ui,
            egui::pos2(stats_x, rect.center().y),
            &format!("+{}", file.additions),
            9.0,
            egui::Color32::from_rgb(78, 190, 116),
            egui::Align2::LEFT_CENTER,
        );
        painter_text(
            ui,
            egui::pos2(stats_x + 32.0, rect.center().y),
            &format!("-{}", file.deletions),
            9.0,
            egui::Color32::from_rgb(230, 92, 92),
            egui::Align2::LEFT_CENTER,
        );
    }

    if hovered {
        let unstage_btn_rect = egui::Rect::from_center_size(
            egui::pos2(rect.right() - 14.0, rect.center().y),
            egui::vec2(18.0, 18.0),
        );
        ui.painter()
            .rect_filled(unstage_btn_rect, 4.0, egui::Color32::from_rgb(58, 58, 58));
        ui.painter().text(
            unstage_btn_rect.center(),
            egui::Align2::CENTER_CENTER,
            X,
            egui::FontId::proportional(10.0),
            egui::Color32::WHITE,
        );
        if ui
            .interact(
                unstage_btn_rect,
                ui.make_persistent_id(format!("unstage_btn_{}", file.path)),
                egui::Sense::click(),
            )
            .clicked()
        {
            state.queue_action(CommitAction::UnstageFile(file.path.clone()));
        }
    }
}

fn file_row_staged_cached(
    ui: &mut egui::Ui,
    file: &CachedFileStatus,
    _muted: egui::Color32,
    state: &mut State,
) {
    let (_, icon_color) = cached_file_icon_for_kind(&file.kind);
    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), FILE_ROW_HEIGHT),
        egui::Sense::click(),
    );

    let hovered = response.hovered();

    if hovered && response.clicked() {
        state.queue_action(CommitAction::UnstageFile(file.path.clone()));
    }
    if hovered {
        ui.painter()
            .rect_filled(rect, 3.0, egui::Color32::from_rgb(48, 48, 48));
    }

    let icon_x = rect.left() + 4.0;
    let icon_rect = egui::Rect::from_center_size(
        egui::pos2(rect.left() + 10.0, rect.center().y),
        egui::vec2(13.0, 13.0),
    );
    crate::ui::core::filetree::paint_file_icon_rect(ui, icon_rect, &file.path, icon_color);

    let path_x = icon_x + 16.0;
    let path_width = rect.width() - 80.0 - (path_x - rect.left());
    let display = truncate_path(&file.path, path_width, 10.0);
    painter_text(
        ui,
        egui::pos2(path_x, rect.center().y),
        &display,
        10.0,
        ui.visuals().text_color(),
        egui::Align2::LEFT_CENTER,
    );

    let stats_x = rect.right() - 72.0;
    if file.additions > 0 || file.deletions > 0 {
        painter_text(
            ui,
            egui::pos2(stats_x, rect.center().y),
            &format!("+{}", file.additions),
            9.0,
            egui::Color32::from_rgb(78, 190, 116),
            egui::Align2::LEFT_CENTER,
        );
        painter_text(
            ui,
            egui::pos2(stats_x + 32.0, rect.center().y),
            &format!("-{}", file.deletions),
            9.0,
            egui::Color32::from_rgb(230, 92, 92),
            egui::Align2::LEFT_CENTER,
        );
    }

    if hovered {
        let unstage_btn_rect = egui::Rect::from_center_size(
            egui::pos2(rect.right() - 14.0, rect.center().y),
            egui::vec2(18.0, 18.0),
        );
        ui.painter()
            .rect_filled(unstage_btn_rect, 4.0, egui::Color32::from_rgb(58, 58, 58));
        ui.painter().text(
            unstage_btn_rect.center(),
            egui::Align2::CENTER_CENTER,
            X,
            egui::FontId::proportional(10.0),
            egui::Color32::WHITE,
        );
        if ui
            .interact(
                unstage_btn_rect,
                ui.make_persistent_id(format!("unstage_cached_btn_{}", file.path)),
                egui::Sense::click(),
            )
            .clicked()
        {
            state.queue_action(CommitAction::UnstageFile(file.path.clone()));
        }
    }
}

fn file_icon_for_kind(kind: &crate::git::models::FileChangeKind) -> (&'static str, egui::Color32) {
    use crate::git::models::FileChangeKind;
    match kind {
        FileChangeKind::Added => (FILE_PLUS, egui::Color32::from_rgb(78, 190, 116)),
        FileChangeKind::Modified => (FILE, egui::Color32::from_rgb(252, 197, 34)),
        FileChangeKind::Deleted => (TRASH, egui::Color32::from_rgb(230, 92, 92)),
        FileChangeKind::Renamed => (ARROW_DOWN, egui::Color32::from_rgb(151, 113, 255)),
        FileChangeKind::TypeChanged => (FOLDER, egui::Color32::from_rgb(172, 172, 172)),
    }
}

fn cached_file_icon_for_kind(kind: &CachedFileChangeKind) -> (&'static str, egui::Color32) {
    match kind {
        CachedFileChangeKind::Added => (FILE_PLUS, egui::Color32::from_rgb(78, 190, 116)),
        CachedFileChangeKind::Modified => (FILE, egui::Color32::from_rgb(252, 197, 34)),
        CachedFileChangeKind::Deleted => (TRASH, egui::Color32::from_rgb(230, 92, 92)),
        CachedFileChangeKind::Renamed => (ARROW_DOWN, egui::Color32::from_rgb(151, 113, 255)),
        CachedFileChangeKind::TypeChanged => (FOLDER, egui::Color32::from_rgb(172, 172, 172)),
    }
}

fn truncate_path(path: &str, max_width: f32, font_size: f32) -> String {
    let char_count = path.chars().count();
    if char_count as f32 * font_size * 0.55 < max_width {
        return path.to_string();
    }

    let parts: Vec<&str> = path.split('/').collect();
    if parts.len() <= 2 {
        let max_chars = (max_width / (font_size * 0.55)) as usize;
        if char_count > max_chars {
            let keep = max_chars.saturating_sub(3);
            let suffix: String = path.chars().skip(char_count.saturating_sub(keep)).collect();
            return format!("...{}", suffix);
        }
        return path.to_string();
    }

    let file_name = parts.last().unwrap();
    let file_char_count = file_name.chars().count();
    let max_chars = (max_width / (font_size * 0.55)) as usize - 4;
    if file_char_count + 4 < max_chars {
        return format!("…/{}", file_name);
    }

    let keep = max_chars.saturating_sub(4);
    let truncated: String = file_name
        .chars()
        .skip(file_char_count.saturating_sub(keep))
        .collect();
    format!("…/…{}", truncated)
}

fn show_discard_confirm(ui: &mut egui::Ui, panel_rect: egui::Rect, state: &mut State) {
    let confirm_rect = panel_rect.shrink2(egui::vec2(20.0, 40.0));
    ui.painter()
        .rect_filled(confirm_rect, 6, egui::Color32::from_rgb(50, 50, 50));
    ui.painter().rect_stroke(
        confirm_rect,
        6,
        egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(100, 100, 100)),
        egui::StrokeKind::Inside,
    );

    let msg_y = confirm_rect.top() + 20.0;
    painter_text(
        ui,
        egui::pos2(confirm_rect.center().x, msg_y),
        "Discard all changes?",
        12.0,
        ui.visuals().text_color(),
        egui::Align2::CENTER_CENTER,
    );

    let btn_y = confirm_rect.bottom() - 18.0;
    let cancel_rect = egui::Rect::from_center_size(
        egui::pos2(confirm_rect.center().x - 40.0, btn_y),
        egui::vec2(60.0, 22.0),
    );
    let confirm_btn_rect = egui::Rect::from_center_size(
        egui::pos2(confirm_rect.center().x + 40.0, btn_y),
        egui::vec2(60.0, 22.0),
    );

    let cancel_resp = ui.interact(
        cancel_rect,
        ui.make_persistent_id("discard_cancel"),
        egui::Sense::click(),
    );
    ui.painter()
        .rect_filled(cancel_rect, 3.0, egui::Color32::from_rgb(60, 60, 60));
    painter_text(
        ui,
        cancel_rect.center(),
        "Cancel",
        10.0,
        ui.visuals().text_color(),
        egui::Align2::CENTER_CENTER,
    );
    if cancel_resp.clicked() {
        state.show_discard_confirm = false;
    }

    let confirm_resp = ui.interact(
        confirm_btn_rect,
        ui.make_persistent_id("discard_confirm"),
        egui::Sense::click(),
    );
    ui.painter()
        .rect_filled(confirm_btn_rect, 3.0, egui::Color32::from_rgb(180, 60, 60));
    painter_text(
        ui,
        confirm_btn_rect.center(),
        "Discard",
        10.0,
        egui::Color32::WHITE,
        egui::Align2::CENTER_CENTER,
    );
    if confirm_resp.clicked() {
        state.show_discard_confirm = false;
        state.queue_action(CommitAction::DiscardAll);
    }
}

fn icon_label(ui: &mut egui::Ui, icon: &str, value: &str, tooltip: &str, muted: egui::Color32) {
    ui.label(egui::RichText::new(icon).size(12.0).color(muted))
        .on_hover_text(tooltip);
    ui.label(egui::RichText::new(value).size(10.0));
}

fn separator(ui: &mut egui::Ui) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(1.0, 12.0), egui::Sense::hover());
    ui.painter().line_segment(
        [rect.center_top(), rect.center_bottom()],
        egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(72, 72, 72)),
    );
}

fn painter_text(
    ui: &egui::Ui,
    pos: egui::Pos2,
    text: &str,
    size: f32,
    color: egui::Color32,
    align: egui::Align2,
) {
    ui.painter()
        .text(pos, align, text, egui::FontId::proportional(size), color);
}
