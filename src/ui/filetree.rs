use eframe::egui;
use egui_phosphor::regular::{
    CARET_DOWN, CARET_RIGHT, FILE, FILE_PLUS, FILE_TEXT, FOLDER, FOLDER_OPEN,
};
use std::collections::BTreeMap;

use crate::git::models::{FileChangeKind, FileStatus};

pub const TREE_ROW_HEIGHT: f32 = 20.0;
pub const TREE_SLOT_WIDTH: f32 = 22.0;
pub const TREE_LEFT_PADDING: f32 = 6.0;
pub const TREE_CARET_SLOT: f32 = 6.0;
pub const TREE_ICON_GAP: f32 = 24.0;

#[derive(Clone, Debug)]
pub enum TreeEntryKind {
    File,
    Directory,
}

#[derive(Clone, Debug)]
pub struct TreeEntry {
    pub path: String,
    pub label: String,
    pub kind: TreeEntryKind,
    pub file_kind: Option<FileChangeKind>,
    pub file_index: Option<usize>,
    pub expanded: bool,
    pub has_children: bool,
    pub children: Vec<TreeEntry>,
}

#[derive(Clone, Debug, Default)]
pub struct TreeState {
    pub rows: Vec<TreeEntry>,
    pub rebuild_key: Option<String>,
}

pub fn paint_tree_tab(
    ui: &mut egui::Ui,
    tree_state: &mut TreeState,
    files: &[FileStatus],
    populated: bool,
    muted: egui::Color32,
    rebuild_key: &str,
    id_salt: &str,
) {
    if !populated {
        ui.label(
            egui::RichText::new("Loading files...")
                .size(10.0)
                .color(muted),
        );
        return;
    }
    if files.is_empty() {
        ui.label(egui::RichText::new("No files").size(10.0).color(muted));
        return;
    }

    rebuild_tree_if_needed(tree_state, files, rebuild_key);
    paint_tree_header(ui, tree_state, muted);
    ui.add_space(6.0);

    egui::ScrollArea::vertical()
        .id_salt(id_salt)
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let len = tree_state.rows.len();
            let mut ancestors_last = Vec::new();
            for (index, row) in tree_state.rows.iter_mut().enumerate() {
                paint_tree_entry(ui, row, 0, &mut ancestors_last, index + 1 == len, muted);
            }
        });
}

fn paint_tree_header(ui: &mut egui::Ui, tree_state: &mut TreeState, muted: egui::Color32) {
    ui.horizontal(|ui| {
        if ui
            .button(egui::RichText::new("Expand All").size(9.0).color(muted))
            .clicked()
        {
            set_all_directories_expanded(tree_state, true);
        }
        if ui
            .button(egui::RichText::new("Collapse All").size(9.0).color(muted))
            .clicked()
        {
            set_all_directories_expanded(tree_state, false);
        }
    });
}

pub fn rebuild_tree_if_needed(tree_state: &mut TreeState, files: &[FileStatus], rebuild_key: &str) {
    if tree_state.rebuild_key.as_deref() == Some(rebuild_key) {
        return;
    }

    tree_state.rows = build_tree_entries(files);
    tree_state.rebuild_key = Some(rebuild_key.to_string());
}

fn build_tree_entries(files: &[FileStatus]) -> Vec<TreeEntry> {
    let mut root_map: BTreeMap<String, TreeEntry> = BTreeMap::new();

    for (file_index, file) in files.iter().enumerate() {
        let segments: Vec<&str> = file
            .path
            .split('/')
            .filter(|segment| !segment.is_empty())
            .collect();
        if segments.is_empty() {
            continue;
        }

        insert_tree_entry(
            &mut root_map,
            &segments,
            0,
            file_index,
            &file.kind,
            String::new(),
        );
    }

    root_map.into_values().collect()
}

fn insert_tree_entry(
    nodes: &mut BTreeMap<String, TreeEntry>,
    segments: &[&str],
    _depth: usize,
    file_index: usize,
    file_kind: &FileChangeKind,
    mut path_prefix: String,
) {
    if !path_prefix.is_empty() {
        path_prefix.push('/');
    }
    path_prefix.push_str(segments[0]);

    let is_file = segments.len() == 1;
    let entry = nodes
        .entry(segments[0].to_string())
        .or_insert_with(|| TreeEntry {
            path: path_prefix.clone(),
            label: segments[0].to_string(),
            kind: if is_file {
                TreeEntryKind::File
            } else {
                TreeEntryKind::Directory
            },
            file_kind: if is_file {
                Some(file_kind.clone())
            } else {
                None
            },
            file_index: if is_file { Some(file_index) } else { None },
            expanded: true,
            has_children: !is_file,
            children: Vec::new(),
        });

    if is_file {
        entry.kind = TreeEntryKind::File;
        entry.file_kind = Some(file_kind.clone());
        entry.file_index = Some(file_index);
        return;
    }

    if segments.len() > 1 {
        let mut child_map: BTreeMap<String, TreeEntry> = entry
            .children
            .drain(..)
            .map(|child| (child.label.clone(), child))
            .collect();
        insert_tree_entry(
            &mut child_map,
            &segments[1..],
            _depth + 1,
            file_index,
            file_kind,
            path_prefix,
        );
        entry.children = child_map.into_values().collect();
        entry.has_children = true;
    }
}

fn paint_tree_entry(
    ui: &mut egui::Ui,
    entry: &mut TreeEntry,
    depth: usize,
    ancestors_last: &mut Vec<bool>,
    is_last: bool,
    muted: egui::Color32,
) -> f32 {
    let row_height = TREE_ROW_HEIGHT;
    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), row_height),
        egui::Sense::click(),
    );

    if response.hovered() {
        ui.painter()
            .rect_filled(rect, 3.0, egui::Color32::from_white_alpha(12));
    }

    let row_left = rect.left() + TREE_LEFT_PADDING;
    let slot_left = row_left + TREE_SLOT_WIDTH * depth as f32;
    let center_y = rect.center().y;

    paint_tree_guides(ui, rect, ancestors_last, muted);

    if matches!(entry.kind, TreeEntryKind::Directory) {
        let caret = if entry.expanded {
            CARET_DOWN
        } else {
            CARET_RIGHT
        };
        ui.painter().text(
            egui::pos2(slot_left + TREE_CARET_SLOT, center_y),
            egui::Align2::CENTER_CENTER,
            caret,
            egui::FontId::proportional(9.0),
            muted,
        );
        if response.clicked() {
            entry.expanded = !entry.expanded;
        }
    }

    let (icon, icon_color) = match entry.kind {
        TreeEntryKind::Directory => {
            let icon = if entry.expanded { FOLDER_OPEN } else { FOLDER };
            (icon, muted)
        }
        TreeEntryKind::File => (
            file_icon(entry.file_kind.as_ref(), &entry.path),
            file_icon_color(entry.file_kind.as_ref()),
        ),
    };

    let icon_x = if matches!(entry.kind, TreeEntryKind::Directory) {
        slot_left + TREE_ICON_GAP
    } else {
        slot_left + 2.0
    };

    ui.painter().text(
        egui::pos2(icon_x, center_y),
        egui::Align2::CENTER_CENTER,
        icon,
        egui::FontId::proportional(12.0),
        icon_color,
    );

    ui.painter().text(
        egui::pos2(icon_x + 10.0, center_y),
        egui::Align2::LEFT_CENTER,
        &entry.label,
        egui::FontId::proportional(10.0),
        ui.visuals().text_color(),
    );

    if let Some(kind) = entry.file_kind.as_ref() {
        let (status_label, status_color) = file_status_label(kind.clone());
        ui.painter().text(
            egui::pos2(rect.right() - 12.0, center_y),
            egui::Align2::RIGHT_CENTER,
            status_label,
            egui::FontId::proportional(9.0),
            status_color,
        );
    }

    let mut subtree_height = row_height;

    if matches!(entry.kind, TreeEntryKind::Directory)
        && entry.expanded
        && !entry.children.is_empty()
    {
        let guide_x = slot_left + TREE_CARET_SLOT;
        let mut child_bottom = rect.bottom();

        ancestors_last.push(is_last);
        let child_len = entry.children.len();
        for (index, child) in entry.children.iter_mut().enumerate() {
            let child_height = paint_tree_entry(
                ui,
                child,
                depth + 1,
                ancestors_last,
                index + 1 == child_len,
                muted,
            );
            child_bottom += child_height;
            subtree_height += child_height;
        }
        ancestors_last.pop();

        ui.painter().line_segment(
            [
                egui::pos2(guide_x, rect.bottom() - 2.0),
                egui::pos2(guide_x, child_bottom - 1.0),
            ],
            egui::Stroke::new(1.0_f32, muted.linear_multiply(0.35)),
        );
    }

    subtree_height
}

fn paint_tree_guides(
    ui: &egui::Ui,
    rect: egui::Rect,
    ancestors_last: &[bool],
    muted: egui::Color32,
) {
    let row_left = rect.left() + TREE_LEFT_PADDING;

    for (depth, is_last) in ancestors_last.iter().enumerate() {
        if *is_last {
            continue;
        }

        let guide_x = row_left + TREE_SLOT_WIDTH * depth as f32 + TREE_CARET_SLOT;
        ui.painter().line_segment(
            [
                egui::pos2(guide_x, rect.top()),
                egui::pos2(guide_x, rect.bottom()),
            ],
            egui::Stroke::new(1.0_f32, muted.linear_multiply(0.28)),
        );
    }
}

fn set_all_directories_expanded(tree_state: &mut TreeState, expanded: bool) {
    for entry in &mut tree_state.rows {
        set_entry_expanded(entry, expanded);
    }
}

fn set_entry_expanded(entry: &mut TreeEntry, expanded: bool) {
    if matches!(entry.kind, TreeEntryKind::Directory) {
        entry.expanded = expanded;
        for child in &mut entry.children {
            set_entry_expanded(child, expanded);
        }
    }
}

fn file_icon(file_kind: Option<&FileChangeKind>, path: &str) -> &'static str {
    match file_kind {
        Some(FileChangeKind::Added) => FILE_PLUS,
        Some(FileChangeKind::Deleted) => FILE_TEXT,
        Some(FileChangeKind::Renamed) => FILE_TEXT,
        Some(FileChangeKind::TypeChanged) => FOLDER,
        Some(FileChangeKind::Modified) | None => file_icon_by_extension(path),
    }
}

fn file_icon_color(file_kind: Option<&FileChangeKind>) -> egui::Color32 {
    match file_kind {
        Some(FileChangeKind::Added) => egui::Color32::from_rgb(78, 190, 116),
        Some(FileChangeKind::Deleted) => egui::Color32::from_rgb(228, 86, 86),
        Some(FileChangeKind::Renamed) => egui::Color32::from_rgb(172, 172, 172),
        Some(FileChangeKind::TypeChanged) => egui::Color32::from_rgb(172, 172, 172),
        Some(FileChangeKind::Modified) | None => egui::Color32::from_rgb(252, 197, 34),
    }
}

fn file_status_label(kind: FileChangeKind) -> (&'static str, egui::Color32) {
    match kind {
        FileChangeKind::Added => ("A", egui::Color32::from_rgb(78, 190, 116)),
        FileChangeKind::Modified => ("M", egui::Color32::from_rgb(252, 197, 34)),
        FileChangeKind::Deleted => ("D", egui::Color32::from_rgb(228, 86, 86)),
        FileChangeKind::Renamed => ("R", egui::Color32::from_rgb(172, 172, 172)),
        FileChangeKind::TypeChanged => ("T", egui::Color32::from_rgb(172, 172, 172)),
    }
}

fn file_icon_by_extension(path: &str) -> &'static str {
    match path.rsplit('.').next() {
        Some("rs") => FILE_TEXT,
        Some("toml") => FILE_TEXT,
        Some("md") => FILE_TEXT,
        Some("json") => FILE_TEXT,
        Some("yaml") | Some("yml") => FILE_TEXT,
        Some("png") | Some("jpg") | Some("jpeg") | Some("gif") | Some("svg") => FILE,
        _ => FILE_TEXT,
    }
}
