use eframe::egui;
use egui_phosphor::regular::{
    ARROW_RIGHT, CARET_DOWN, CARET_RIGHT, FILE, FILE_PLUS, FILE_TEXT, FOLDER,
};
use std::collections::HashSet;
use std::hash::Hash;

const LEFT_PANEL_WIDTH_RATIO: f32 = 0.20;
const LEFT_PANEL_MIN_WIDTH: f32 = 180.0;
const LEFT_PANEL_MAX_WIDTH: f32 = 320.0;

const HUNK_ROW_HEIGHT: f32 = 20.0;
const LINE_ROW_HEIGHT: f32 = 18.0;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum CommitDiffFileKind {
    Added,
    Modified,
    Deleted,
    Renamed,
    TypeChanged,
    Copied,
    Untracked,
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum CommitDiffLineKind {
    Context,
    Addition,
    Deletion,
    Binary,
    EofAddition,
    EofDeletion,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct CommitDiffLine {
    pub old_lineno: Option<u32>,
    pub new_lineno: Option<u32>,
    pub kind: CommitDiffLineKind,
    pub content: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct CommitDiffHunk {
    pub header: String,
    pub old_start: u32,
    pub old_lines: u32,
    pub new_start: u32,
    pub new_lines: u32,
    pub lines: Vec<CommitDiffLine>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct CommitDiffFile {
    pub path: String,
    pub old_path: Option<String>,
    pub kind: CommitDiffFileKind,
    pub staged: bool,
    pub additions: usize,
    pub deletions: usize,
    pub is_binary: bool,
    pub hunks: Vec<CommitDiffHunk>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct CommitDiffSummary {
    pub files_changed: usize,
    pub additions: usize,
    pub deletions: usize,
    pub hunks: usize,
    pub lines: usize,
    pub truncated: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct CommitDiffViewModel {
    pub commit_hash: String,
    pub files: Vec<CommitDiffFile>,
    pub summary: CommitDiffSummary,
}

#[derive(Default)]
pub struct DiffTimelineState {
    selected_file_path: Option<String>,
    pub collapsed_files: HashSet<String>,
    pub collapsed_hunks: HashSet<(String, usize)>,
}

impl DiffTimelineState {
    pub fn selected_file_path(&self) -> Option<&str> {
        self.selected_file_path.as_deref()
    }

    pub fn select_file_path(&mut self, path: Option<String>) {
        self.selected_file_path = path;
    }
}

#[derive(Copy, Clone)]
struct IconRenderer<'a> {
    ptr: *mut (dyn FnMut(&mut egui::Ui, egui::Rect, &str, &CommitDiffFileKind, egui::Color32) + 'a),
}

impl<'a> IconRenderer<'a> {
    fn call(
        &mut self,
        ui: &mut egui::Ui,
        rect: egui::Rect,
        path: &str,
        kind: &CommitDiffFileKind,
        color: egui::Color32,
    ) {
        unsafe {
            (*self.ptr)(ui, rect, path, kind, color);
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn show(
    ui: &mut egui::Ui,
    state: &mut DiffTimelineState,
    model: Option<&CommitDiffViewModel>,
    render_icon: Option<
        &mut dyn FnMut(&mut egui::Ui, egui::Rect, &str, &CommitDiffFileKind, egui::Color32),
    >,
) {
    let muted = egui::Color32::from_rgb(172, 172, 172);
    let accent = egui::Color32::from_rgb(78, 190, 116);

    ui.horizontal(|ui| {
        if let Some(model) = model {
            ui.label(
                egui::RichText::new(format!("{} files", model.files.len()))
                    .size(10.0)
                    .color(muted),
            );
            ui.label(
                egui::RichText::new(format!("+{}", model.summary.additions))
                    .size(10.0)
                    .color(accent),
            );
            ui.label(
                egui::RichText::new(format!("-{}", model.summary.deletions))
                    .size(10.0)
                    .color(egui::Color32::from_rgb(230, 92, 92)),
            );
            ui.label(
                egui::RichText::new(format!("{} hunks", model.summary.hunks))
                    .size(10.0)
                    .color(muted),
            );
            if model.summary.truncated {
                ui.label(
                    egui::RichText::new("truncated")
                        .size(10.0)
                        .color(egui::Color32::from_rgb(252, 197, 34)),
                );
            }
        } else {
            ui.label(
                egui::RichText::new("Loading diff details...")
                    .size(10.0)
                    .color(muted),
            );
        }
    });

    ui.add_space(8.0);

    let Some(model) = model else {
        return;
    };

    if state.selected_file_path.is_none() {
        state.select_file_path(model.files.first().map(|file| file.path.clone()));
    }

    if model.files.is_empty() {
        ui.label(
            egui::RichText::new("No diff data for this commit")
                .size(10.0)
                .color(muted),
        );
        return;
    }

    let renderer = render_icon.map(|f| IconRenderer { ptr: f as *mut _ });

    let total_width = ui.available_width();
    let left_width = (total_width * LEFT_PANEL_WIDTH_RATIO)
        .clamp(LEFT_PANEL_MIN_WIDTH, LEFT_PANEL_MAX_WIDTH)
        .min(total_width * 0.45);
    let right_width = (total_width - left_width - 12.0).max(0.0);
    let content_height = ui.available_height();

    ui.horizontal(|ui| {
        ui.allocate_ui_with_layout(
            egui::vec2(left_width, content_height),
            egui::Layout::top_down(egui::Align::Min),
            |ui| paint_file_list(ui, state, model, muted, renderer),
        );

        ui.add_space(12.0);

        ui.allocate_ui_with_layout(
            egui::vec2(right_width, content_height),
            egui::Layout::top_down(egui::Align::Min),
            |ui| paint_timeline(ui, state, model, muted, renderer),
        );
    });
}

fn paint_file_list(
    ui: &mut egui::Ui,
    state: &mut DiffTimelineState,
    model: &CommitDiffViewModel,
    muted: egui::Color32,
    mut render_icon: Option<IconRenderer<'_>>,
) {
    ui.label(egui::RichText::new("Files").size(11.0).strong());
    ui.add_space(6.0);

    egui::ScrollArea::vertical()
        .id_salt("commit_diff_file_list_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            for file in &model.files {
                let (rect, response) = ui.allocate_exact_size(
                    egui::vec2(ui.available_width(), 22.0),
                    egui::Sense::click(),
                );

                let selected = state
                    .selected_file_path()
                    .is_some_and(|path| path == file.path);
                if selected {
                    ui.painter()
                        .rect_filled(rect, 3.0, egui::Color32::from_white_alpha(16));
                } else if response.hovered() {
                    ui.painter()
                        .rect_filled(rect, 3.0, egui::Color32::from_white_alpha(8));
                }

                let icon_color = file_icon_for(file).1;
                let icon_rect = egui::Rect::from_center_size(
                    egui::pos2(rect.left() + 10.0, rect.center().y),
                    egui::vec2(13.0, 13.0),
                );
                if let Some(ref mut render_icon) = render_icon {
                    render_icon.call(ui, icon_rect, &file.path, &file.kind, icon_color);
                } else {
                    let (icon, icon_color) = file_icon_for(file);
                    ui.painter().text(
                        egui::pos2(rect.left() + 4.0, rect.center().y),
                        egui::Align2::LEFT_CENTER,
                        icon,
                        egui::FontId::proportional(11.0),
                        icon_color,
                    );
                }

                ui.painter().text(
                    egui::pos2(rect.left() + 18.0, rect.center().y),
                    egui::Align2::LEFT_CENTER,
                    file_display_name(file),
                    egui::FontId::proportional(9.5),
                    ui.visuals().text_color(),
                );
                ui.painter().text(
                    egui::pos2(rect.right() - 4.0, rect.center().y),
                    egui::Align2::RIGHT_CENTER,
                    format!(
                        "{} +{} -{}",
                        file_status_label(&file.kind),
                        file.additions,
                        file.deletions
                    ),
                    egui::FontId::monospace(8.8),
                    muted,
                );

                if response.clicked() {
                    state.select_file_path(Some(file.path.clone()));
                }
            }
        });
}

fn paint_timeline(
    ui: &mut egui::Ui,
    state: &mut DiffTimelineState,
    model: &CommitDiffViewModel,
    muted: egui::Color32,
    render_icon: Option<IconRenderer<'_>>,
) {
    ui.label(egui::RichText::new("Diff timeline").size(11.0).strong());
    ui.add_space(6.0);

    egui::ScrollArea::vertical()
        .id_salt("commit_diff_timeline_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            for file in &model.files {
                paint_file_header_row(ui, state, file, muted, render_icon);
                ui.add_space(4.0);
                let file_open = !state.collapsed_files.contains(&file.path);
                if file_open {
                    for (hunk_index, hunk) in file.hunks.iter().enumerate() {
                        paint_hunk_row(ui, state, file, hunk_index, hunk, muted);
                        if !state
                            .collapsed_hunks
                            .contains(&(file.path.clone(), hunk_index))
                        {
                            paint_lines(ui, hunk, muted);
                        }
                        ui.add_space(6.0);
                    }
                }
                ui.add_space(8.0);
            }
        });
}

fn paint_file_header_row(
    ui: &mut egui::Ui,
    state: &mut DiffTimelineState,
    file: &CommitDiffFile,
    muted: egui::Color32,
    mut render_icon: Option<IconRenderer<'_>>,
) {
    let selected = state
        .selected_file_path()
        .is_some_and(|path| path == file.path);
    let open = !state.collapsed_files.contains(&file.path);

    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(ui.available_width(), 22.0), egui::Sense::click());
    let fill = if selected {
        egui::Color32::from_rgb(54, 54, 54)
    } else {
        egui::Color32::from_rgb(44, 44, 44)
    };
    ui.painter().rect_filled(rect, 2.0, fill);

    let caret = if open { CARET_DOWN } else { CARET_RIGHT };
    ui.painter().text(
        egui::pos2(rect.left() + 8.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        caret,
        egui::FontId::proportional(10.0),
        muted,
    );

    let icon_color = file_icon_for(file).1;
    let icon_rect = egui::Rect::from_center_size(
        egui::pos2(rect.left() + 26.0, rect.center().y),
        egui::vec2(13.0, 13.0),
    );
    if let Some(ref mut render_icon) = render_icon {
        render_icon.call(ui, icon_rect, &file.path, &file.kind, icon_color);
    } else {
        let (icon, icon_color) = file_icon_for(file);
        ui.painter().text(
            egui::pos2(rect.left() + 20.0, rect.center().y),
            egui::Align2::LEFT_CENTER,
            icon,
            egui::FontId::proportional(10.5),
            icon_color,
        );
    }

    ui.painter().text(
        egui::pos2(rect.left() + 36.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        file_display_name(file),
        egui::FontId::proportional(9.5),
        ui.visuals().text_color(),
    );
    ui.painter().text(
        egui::pos2(rect.right() - 6.0, rect.center().y),
        egui::Align2::RIGHT_CENTER,
        format!(
            "{} +{} -{}",
            file_status_label(&file.kind),
            file.additions,
            file.deletions
        ),
        egui::FontId::monospace(8.8),
        file_color(&file.kind),
    );

    if response.clicked() {
        if open {
            state.collapsed_files.insert(file.path.clone());
            state.collapsed_hunks.retain(|(path, _)| path != &file.path);
        } else {
            state.collapsed_files.remove(&file.path);
        }
    }
}

fn paint_hunk_row(
    ui: &mut egui::Ui,
    state: &mut DiffTimelineState,
    file: &CommitDiffFile,
    hunk_index: usize,
    hunk: &CommitDiffHunk,
    muted: egui::Color32,
) {
    let key = (file.path.clone(), hunk_index);
    let open = !state.collapsed_hunks.contains(&key);
    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), HUNK_ROW_HEIGHT),
        egui::Sense::click(),
    );

    ui.painter()
        .rect_filled(rect, 0.0, egui::Color32::from_rgb(34, 34, 34));
    let caret = if open { CARET_DOWN } else { CARET_RIGHT };
    ui.painter().text(
        egui::pos2(rect.left() + 8.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        caret,
        egui::FontId::proportional(9.5),
        muted,
    );
    ui.painter().text(
        egui::pos2(rect.left() + 24.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        &hunk.header,
        egui::FontId::monospace(9.2),
        ui.visuals().text_color(),
    );
    ui.painter().text(
        egui::pos2(rect.right() - 6.0, rect.center().y),
        egui::Align2::RIGHT_CENTER,
        format!("{} lines", hunk.lines.len()),
        egui::FontId::monospace(8.6),
        muted,
    );

    if response.clicked() {
        if open {
            state.collapsed_hunks.insert(key);
        } else {
            state.collapsed_hunks.remove(&key);
        }
    }
}

fn paint_lines(ui: &mut egui::Ui, hunk: &CommitDiffHunk, muted: egui::Color32) {
    for line in &hunk.lines {
        let (rect, _) = ui.allocate_exact_size(
            egui::vec2(ui.available_width(), LINE_ROW_HEIGHT),
            egui::Sense::hover(),
        );
        let fill = match line.kind {
            CommitDiffLineKind::Addition | CommitDiffLineKind::EofAddition => {
                egui::Color32::from_rgba_unmultiplied(58, 129, 72, 56)
            }
            CommitDiffLineKind::Deletion | CommitDiffLineKind::EofDeletion => {
                egui::Color32::from_rgba_unmultiplied(157, 62, 62, 56)
            }
            CommitDiffLineKind::Binary => egui::Color32::from_rgb(38, 38, 38),
            CommitDiffLineKind::Context => egui::Color32::from_rgb(29, 29, 29),
        };
        ui.painter().rect_filled(rect, 0.0, fill);
        ui.painter().text(
            egui::pos2(rect.left() + 6.0, rect.center().y),
            egui::Align2::LEFT_CENTER,
            line_prefix(&line.kind),
            egui::FontId::monospace(9.0),
            line_prefix_color(&line.kind),
        );
        ui.painter().text(
            egui::pos2(rect.left() + 22.0, rect.center().y),
            egui::Align2::LEFT_CENTER,
            line_number(line.old_lineno),
            egui::FontId::monospace(8.6),
            muted,
        );
        ui.painter().text(
            egui::pos2(rect.left() + 50.0, rect.center().y),
            egui::Align2::LEFT_CENTER,
            line_number(line.new_lineno),
            egui::FontId::monospace(8.6),
            muted,
        );
        ui.painter().text(
            egui::pos2(rect.left() + 82.0, rect.center().y),
            egui::Align2::LEFT_CENTER,
            &line.content,
            egui::FontId::monospace(8.9),
            line_text_color(&line.kind, ui.visuals().text_color()),
        );
    }
}

fn line_prefix(kind: &CommitDiffLineKind) -> &'static str {
    match kind {
        CommitDiffLineKind::Addition | CommitDiffLineKind::EofAddition => "+",
        CommitDiffLineKind::Deletion | CommitDiffLineKind::EofDeletion => "-",
        CommitDiffLineKind::Binary => "B",
        CommitDiffLineKind::Context => " ",
    }
}

fn line_prefix_color(kind: &CommitDiffLineKind) -> egui::Color32 {
    match kind {
        CommitDiffLineKind::Addition | CommitDiffLineKind::EofAddition => {
            egui::Color32::from_rgb(78, 190, 116)
        }
        CommitDiffLineKind::Deletion | CommitDiffLineKind::EofDeletion => {
            egui::Color32::from_rgb(230, 92, 92)
        }
        _ => egui::Color32::from_rgb(172, 172, 172),
    }
}

fn line_number(value: Option<u32>) -> String {
    value
        .map(|n| format!("{:>4}", n))
        .unwrap_or_else(|| "    ".to_string())
}

fn line_text_color(kind: &CommitDiffLineKind, base: egui::Color32) -> egui::Color32 {
    match kind {
        CommitDiffLineKind::Addition | CommitDiffLineKind::EofAddition => {
            egui::Color32::from_rgb(232, 255, 236)
        }
        CommitDiffLineKind::Deletion | CommitDiffLineKind::EofDeletion => {
            egui::Color32::from_rgb(255, 235, 235)
        }
        _ => base,
    }
}

fn file_icon_for(file: &CommitDiffFile) -> (&'static str, egui::Color32) {
    match file.kind {
        CommitDiffFileKind::Added | CommitDiffFileKind::Copied | CommitDiffFileKind::Untracked => {
            (FILE_PLUS, egui::Color32::from_rgb(78, 190, 116))
        }
        CommitDiffFileKind::Deleted => (FILE_TEXT, egui::Color32::from_rgb(230, 92, 92)),
        CommitDiffFileKind::Renamed => (FILE, egui::Color32::from_rgb(151, 113, 255)),
        CommitDiffFileKind::TypeChanged => (FOLDER, egui::Color32::from_rgb(172, 172, 172)),
        CommitDiffFileKind::Modified | CommitDiffFileKind::Unknown => {
            (FILE, egui::Color32::from_rgb(252, 197, 34))
        }
    }
}

fn file_display_name(file: &CommitDiffFile) -> String {
    if let Some(old_path) = &file.old_path {
        format!("{} {} {}", old_path, ARROW_RIGHT, file.path)
    } else {
        file.path.clone()
    }
}

fn file_color(kind: &CommitDiffFileKind) -> egui::Color32 {
    match kind {
        CommitDiffFileKind::Added | CommitDiffFileKind::Copied | CommitDiffFileKind::Untracked => {
            egui::Color32::from_rgb(78, 190, 116)
        }
        CommitDiffFileKind::Deleted => egui::Color32::from_rgb(230, 92, 92),
        CommitDiffFileKind::Renamed => egui::Color32::from_rgb(151, 113, 255),
        CommitDiffFileKind::TypeChanged => egui::Color32::from_rgb(172, 172, 172),
        CommitDiffFileKind::Modified | CommitDiffFileKind::Unknown => {
            egui::Color32::from_rgb(252, 197, 34)
        }
    }
}

fn file_status_label(kind: &CommitDiffFileKind) -> &'static str {
    match kind {
        CommitDiffFileKind::Added | CommitDiffFileKind::Copied | CommitDiffFileKind::Untracked => {
            "A"
        }
        CommitDiffFileKind::Modified | CommitDiffFileKind::Unknown => "M",
        CommitDiffFileKind::Deleted => "D",
        CommitDiffFileKind::Renamed => "R",
        CommitDiffFileKind::TypeChanged => "T",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_model() -> CommitDiffViewModel {
        CommitDiffViewModel {
            commit_hash: "abc123".to_string(),
            summary: CommitDiffSummary {
                files_changed: 1,
                additions: 3,
                deletions: 1,
                hunks: 1,
                lines: 4,
                truncated: false,
            },
            files: vec![CommitDiffFile {
                path: "src/main.rs".to_string(),
                old_path: None,
                kind: CommitDiffFileKind::Modified,
                staged: false,
                additions: 3,
                deletions: 1,
                is_binary: false,
                hunks: vec![CommitDiffHunk {
                    header: "@@ -1,2 +1,4 @@".to_string(),
                    old_start: 1,
                    old_lines: 2,
                    new_start: 1,
                    new_lines: 4,
                    lines: vec![
                        CommitDiffLine {
                            old_lineno: Some(1),
                            new_lineno: Some(1),
                            kind: CommitDiffLineKind::Context,
                            content: "fn main() {".to_string(),
                        },
                        CommitDiffLine {
                            old_lineno: None,
                            new_lineno: Some(2),
                            kind: CommitDiffLineKind::Addition,
                            content: "+println!(\"hi\");".to_string(),
                        },
                    ],
                }],
            }],
        }
    }

    #[test]
    fn selected_file_can_be_set() {
        let mut state = DiffTimelineState::default();
        state.select_file_path(Some("src/main.rs".to_string()));
        assert_eq!(state.selected_file_path(), Some("src/main.rs"));
    }

    #[test]
    fn file_icons_and_labels_work() {
        let model = sample_model();
        assert_eq!(file_status_label(&model.files[0].kind), "M");
        assert!(file_display_name(&model.files[0]).contains("src/main.rs"));
    }
}
