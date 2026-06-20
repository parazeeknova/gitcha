use crate::state::AppState;
use crate::ui::body::CommitDrawerLayout;
use crate::ui::command_palette::QuickLaunchAction;
use eframe::egui;
use egui_phosphor::regular::{
    ARROW_COUNTER_CLOCKWISE, ARROW_LINE_DOWN, ARROW_LINE_UP, ARROWS_CLOCKWISE, BROWSERS,
    CARET_DOWN, CHECK, COLUMNS, FOLDER, GIT_BRANCH, GIT_FORK, ROWS, SIDEBAR, STACK,
    TERMINAL_WINDOW, TEXT_ALIGN_LEFT,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ToolbarAction {
    None,
    QuickLaunch,
    Fetch,
    Pull,
    Push,
    StashSave,
    StashApply,
    StashPop,
    NewBranch,
    SetDrawerLayoutHorizontal,
    SetDrawerLayoutVertical,
    ToggleTerminal,
}

const TOOLBAR_HEIGHT: f32 = 56.0;
const CENTER_WIDTH: f32 = 230.0;
const ACTION_WIDTH: f32 = 58.0;
const QUICK_ACTION_WIDTH: f32 = 76.0;
const ACTION_HEIGHT: f32 = 48.0;
const LEFT_ACTIONS: f32 = QUICK_ACTION_WIDTH + ACTION_WIDTH * 4.0;
const RIGHT_ACTIONS: f32 = ACTION_WIDTH * 6.0;

pub fn show(
    ui: &mut egui::Ui,
    repo_name: Option<&str>,
    current_branch: Option<&str>,
    state: &AppState,
    current_repo_owned_by_authed_user: Option<bool>,
    current_layout: CommitDrawerLayout,
    busy_action: Option<QuickLaunchAction>,
) -> ToolbarAction {
    let width = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width, TOOLBAR_HEIGHT), egui::Sense::hover());

    let visuals = ui.visuals().widgets.inactive;
    let stroke = egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(72, 72, 72));
    let top_edge_stroke = egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(78, 78, 78));
    ui.painter().rect_filled(rect, 0.0, visuals.bg_fill);
    ui.painter()
        .line_segment([rect.left_top(), rect.right_top()], top_edge_stroke);
    ui.painter()
        .line_segment([rect.left_bottom(), rect.right_bottom()], stroke);

    let (left_rect, center_rect, right_rect) = section_rects(rect);

    let mut toolbar_action = ToolbarAction::None;

    child_ui(
        ui,
        left_rect.shrink2(egui::vec2(8.0, 0.0)),
        "toolbar_left",
        egui::Layout::left_to_right(egui::Align::Center),
        |ui| left_panel(ui, &mut toolbar_action, busy_action.as_ref()),
    );
    child_ui(
        ui,
        right_rect.shrink2(egui::vec2(8.0, 0.0)),
        "toolbar_right",
        egui::Layout::right_to_left(egui::Align::Center),
        |ui| {
            right_panel(
                ui,
                &mut toolbar_action,
                current_layout,
                busy_action.as_ref(),
            )
        },
    );

    // Draw center panel background and borders last so they render on top of left/right panels
    let center_hovered = repo_name.is_some() && ui.rect_contains_pointer(center_rect);
    let center_fill = if center_hovered {
        egui::Color32::from_rgb(52, 52, 52)
    } else {
        egui::Color32::from_rgb(43, 43, 43)
    };
    ui.painter().rect_filled(center_rect, 0.0, center_fill);
    ui.painter()
        .line_segment([center_rect.left_top(), center_rect.left_bottom()], stroke);
    ui.painter().line_segment(
        [center_rect.right_top(), center_rect.right_bottom()],
        stroke,
    );

    child_ui(
        ui,
        center_rect.shrink2(egui::vec2(8.0, 0.0)),
        "toolbar_center",
        egui::Layout::left_to_right(egui::Align::Center),
        |ui| {
            center_panel(
                ui,
                repo_name,
                current_branch,
                state,
                current_repo_owned_by_authed_user,
            )
        },
    );
    toolbar_action
}

fn section_rects(rect: egui::Rect) -> (egui::Rect, egui::Rect, egui::Rect) {
    let center_width = CENTER_WIDTH.min((rect.width() * 0.32).max(180.0));
    let preferred_left = rect.center().x - center_width * 0.5;
    let min_left = rect.left() + LEFT_ACTIONS.min(rect.width() * 0.36);
    let max_left = rect.right() - RIGHT_ACTIONS.min(rect.width() * 0.42) - center_width;
    let center_left = if min_left <= max_left {
        preferred_left.clamp(min_left, max_left)
    } else {
        preferred_left.clamp(rect.left(), rect.right() - center_width)
    };

    let center_rect = egui::Rect::from_min_size(
        egui::pos2(center_left, rect.top()),
        egui::vec2(center_width, rect.height()),
    );
    let left_rect = egui::Rect::from_min_max(rect.left_top(), center_rect.left_bottom());
    let right_rect = egui::Rect::from_min_max(center_rect.right_top(), rect.right_bottom());

    (left_rect, center_rect, right_rect)
}

fn child_ui<R>(
    ui: &mut egui::Ui,
    rect: egui::Rect,
    id_salt: &'static str,
    layout: egui::Layout,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> egui::InnerResponse<R> {
    ui.scope_builder(
        egui::UiBuilder::new()
            .id_salt(id_salt)
            .max_rect(rect)
            .layout(layout),
        add_contents,
    )
}

fn left_panel(
    ui: &mut egui::Ui,
    action: &mut ToolbarAction,
    busy_action: Option<&QuickLaunchAction>,
) {
    ui.spacing_mut().item_spacing = egui::vec2(6.0, 0.0);
    let toolbar_enabled = busy_action.is_none();
    if toolbar_button(
        ui,
        ToolbarButtonArgs {
            width: QUICK_ACTION_WIDTH,
            icon: FOLDER,
            label: "Quick Launch",
            suffix: None,
            enabled: toolbar_enabled,
            busy: false,
        },
    ) {
        *action = ToolbarAction::QuickLaunch;
    }
    if toolbar_button(
        ui,
        ToolbarButtonArgs {
            width: ACTION_WIDTH,
            icon: ARROW_COUNTER_CLOCKWISE,
            label: "Fetch",
            suffix: None,
            enabled: toolbar_enabled,
            busy: busy_action.is_some_and(|busy| busy == &QuickLaunchAction::Fetch),
        },
    ) {
        *action = ToolbarAction::Fetch;
    }
    if toolbar_button(
        ui,
        ToolbarButtonArgs {
            width: ACTION_WIDTH,
            icon: ARROW_LINE_DOWN,
            label: "Pull",
            suffix: None,
            enabled: toolbar_enabled,
            busy: busy_action.is_some_and(|busy| busy == &QuickLaunchAction::Pull),
        },
    ) {
        *action = ToolbarAction::Pull;
    }
    if toolbar_button(
        ui,
        ToolbarButtonArgs {
            width: ACTION_WIDTH,
            icon: ARROW_LINE_UP,
            label: "Push",
            suffix: None,
            enabled: toolbar_enabled,
            busy: busy_action.is_some_and(|busy| busy == &QuickLaunchAction::Push),
        },
    ) {
        *action = ToolbarAction::Push;
    }
    toolbar_menu_button(
        ui,
        ToolbarMenuButtonArgs {
            width: ACTION_WIDTH,
            icon: STACK,
            label: "Stash",
            suffix: Some(CARET_DOWN),
            enabled: toolbar_enabled,
            busy: false,
        },
        |ui| {
            if ui.button("Stash changes").clicked() {
                *action = ToolbarAction::StashSave;
            }
            if ui.button("Apply stash").clicked() {
                *action = ToolbarAction::StashApply;
            }
            if ui.button("Pop stash").clicked() {
                *action = ToolbarAction::StashPop;
            }
        },
    );
}

fn center_panel(
    ui: &mut egui::Ui,
    repo_name: Option<&str>,
    current_branch: Option<&str>,
    state: &AppState,
    current_repo_owned_by_authed_user: Option<bool>,
) {
    let rect = ui.max_rect();
    let group_rect = egui::Rect::from_center_size(rect.center(), egui::vec2(200.0, ACTION_HEIGHT));
    let text_rect = egui::Rect::from_min_size(
        egui::pos2(group_rect.left() + 8.0, group_rect.top()),
        egui::vec2(184.0, ACTION_HEIGHT),
    );

    let menu_icon_rect = egui::Rect::from_min_size(
        egui::pos2(rect.left(), rect.top() + (rect.height() - 16.0) / 2.0),
        egui::vec2(16.0, 16.0),
    );

    if repo_name.is_some() {
        let btn_resp = ui
            .put(
                menu_icon_rect,
                egui::Button::new(egui::RichText::new(TEXT_ALIGN_LEFT).size(14.0))
                    .frame(false)
                    .min_size(egui::vec2(16.0, 16.0)),
            )
            .on_hover_text("Repository Details");

        let response = ui
            .interact(rect, ui.id().with("repo_details_btn"), egui::Sense::click())
            .on_hover_text("Repository Details")
            .on_hover_cursor(egui::CursorIcon::PointingHand);

        if btn_resp.clicked() {
            egui::Popup::toggle_id(ui.ctx(), response.id);
        }

        egui::Popup::menu(&response)
            .align(egui::RectAlign::BOTTOM)
            .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
            .show(|ui| {
                let t = ui.ctx().animate_bool(response.id.with("popup_anim"), true);
                ui.multiply_opacity(t);
                ui.add_space(t * 6.0);

                crate::gh::ui::gh_dropdown::show(
                    ui,
                    repo_name,
                    current_branch,
                    state,
                    current_repo_owned_by_authed_user,
                );
            });
    } else {
        ui.painter().text(
            menu_icon_rect.center(),
            egui::Align2::CENTER_CENTER,
            TEXT_ALIGN_LEFT,
            egui::FontId::proportional(14.0),
            ui.visuals().text_color(),
        );
    }

    child_ui(
        ui,
        text_rect,
        "toolbar_center_text",
        egui::Layout::top_down(egui::Align::Center),
        |ui| {
            ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);
            if let Some(name) = repo_name {
                ui.add(
                    egui::Label::new(egui::RichText::new(name).size(13.0).strong())
                        .truncate()
                        .halign(egui::Align::Center),
                );
                let branch_name = current_branch.unwrap_or("no branch");
                let branch_text = format!("{} {}", GIT_BRANCH, branch_name);
                ui.add_space(3.0);
                let rich_text = egui::RichText::new(branch_text).size(10.0);
                ui.add(
                    egui::Label::new(rich_text)
                        .truncate()
                        .halign(egui::Align::Center),
                );
            } else {
                ui.add(
                    egui::Label::new(
                        egui::RichText::new("Welcome to Palimpsest!")
                            .size(12.0)
                            .strong(),
                    )
                    .truncate()
                    .halign(egui::Align::Center),
                );
                ui.add(
                    egui::Label::new(
                        egui::RichText::new("Open a repo to start")
                            .size(10.0)
                            .color(egui::Color32::from_rgb(140, 140, 140)),
                    )
                    .truncate()
                    .halign(egui::Align::Center),
                );
            }
        },
    );
}

fn right_panel(
    ui: &mut egui::Ui,
    action: &mut ToolbarAction,
    current_layout: CommitDrawerLayout,
    busy_action: Option<&QuickLaunchAction>,
) {
    ui.spacing_mut().item_spacing = egui::vec2(6.0, 0.0);
    let toolbar_enabled = busy_action.is_none();
    toolbar_button(
        ui,
        ToolbarButtonArgs {
            width: ACTION_WIDTH,
            icon: BROWSERS,
            label: "Workspace",
            suffix: Some(CARET_DOWN),
            enabled: toolbar_enabled,
            busy: false,
        },
    );
    toolbar_menu_button(
        ui,
        ToolbarMenuButtonArgs {
            width: ACTION_WIDTH,
            icon: SIDEBAR,
            label: "Appearance",
            suffix: Some(CARET_DOWN),
            enabled: toolbar_enabled,
            busy: false,
        },
        |ui| {
            ui.spacing_mut().item_spacing = egui::vec2(0.0, 1.0);
            ui.label(
                egui::RichText::new("Commit Drawer")
                    .size(10.0)
                    .color(egui::Color32::from_rgb(120, 120, 120)),
            );
            ui.add_space(2.0);

            let is_horizontal = current_layout == CommitDrawerLayout::Horizontal;
            let is_vertical = current_layout == CommitDrawerLayout::Vertical;

            let row_width = 140.0;
            let row_height = 20.0;

            // Horizontal option
            let (h_rect, h_resp) =
                ui.allocate_exact_size(egui::vec2(row_width, row_height), egui::Sense::click());
            if h_resp.hovered() {
                ui.painter()
                    .rect_filled(h_rect, 3.0, egui::Color32::from_white_alpha(15));
            }
            let text_color = if is_horizontal {
                egui::Color32::WHITE
            } else {
                egui::Color32::from_rgb(190, 190, 190)
            };
            ui.painter().text(
                egui::pos2(h_rect.left() + 6.0, h_rect.center().y),
                egui::Align2::LEFT_CENTER,
                format!("{} Horizontal", ROWS),
                egui::FontId::proportional(11.0),
                text_color,
            );
            if is_horizontal {
                ui.painter().text(
                    egui::pos2(h_rect.right() - 6.0, h_rect.center().y),
                    egui::Align2::RIGHT_CENTER,
                    CHECK,
                    egui::FontId::proportional(11.0),
                    egui::Color32::from_rgb(130, 200, 130),
                );
            }
            if h_resp.clicked() {
                *action = ToolbarAction::SetDrawerLayoutHorizontal;
                ui.close();
            }

            // Vertical option
            let (v_rect, v_resp) =
                ui.allocate_exact_size(egui::vec2(row_width, row_height), egui::Sense::click());
            if v_resp.hovered() {
                ui.painter()
                    .rect_filled(v_rect, 3.0, egui::Color32::from_white_alpha(15));
            }
            let text_color = if is_vertical {
                egui::Color32::WHITE
            } else {
                egui::Color32::from_rgb(190, 190, 190)
            };
            ui.painter().text(
                egui::pos2(v_rect.left() + 6.0, v_rect.center().y),
                egui::Align2::LEFT_CENTER,
                format!("{} Vertical", COLUMNS),
                egui::FontId::proportional(11.0),
                text_color,
            );
            if is_vertical {
                ui.painter().text(
                    egui::pos2(v_rect.right() - 6.0, v_rect.center().y),
                    egui::Align2::RIGHT_CENTER,
                    CHECK,
                    egui::FontId::proportional(11.0),
                    egui::Color32::from_rgb(130, 200, 130),
                );
            }
            if v_resp.clicked() {
                *action = ToolbarAction::SetDrawerLayoutVertical;
                ui.close();
            }
        },
    );
    if toolbar_button(
        ui,
        ToolbarButtonArgs {
            width: ACTION_WIDTH,
            icon: TERMINAL_WINDOW,
            label: "Console",
            suffix: None,
            enabled: toolbar_enabled,
            busy: false,
        },
    ) {
        *action = ToolbarAction::ToggleTerminal;
    }
    toolbar_button(
        ui,
        ToolbarButtonArgs {
            width: ACTION_WIDTH,
            icon: ARROWS_CLOCKWISE,
            label: "Open in",
            suffix: Some(CARET_DOWN),
            enabled: toolbar_enabled,
            busy: false,
        },
    );
    if toolbar_button(
        ui,
        ToolbarButtonArgs {
            width: ACTION_WIDTH,
            icon: GIT_FORK,
            label: "New Branch",
            suffix: None,
            enabled: toolbar_enabled,
            busy: false,
        },
    ) {
        *action = ToolbarAction::NewBranch;
    }
}

struct ToolbarButtonArgs<'a> {
    width: f32,
    icon: &'a str,
    label: &'a str,
    suffix: Option<&'a str>,
    enabled: bool,
    busy: bool,
}

fn toolbar_button(ui: &mut egui::Ui, args: ToolbarButtonArgs<'_>) -> bool {
    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(args.width, ACTION_HEIGHT),
        if args.enabled {
            egui::Sense::click()
        } else {
            egui::Sense::hover()
        },
    );

    if args.enabled && response.hovered() {
        ui.painter()
            .rect_filled(rect, 4.0, egui::Color32::from_white_alpha(18));
    }

    let text_color = if args.enabled {
        ui.visuals().text_color()
    } else {
        ui.visuals().widgets.noninteractive.text_color()
    };

    if args.busy {
        let spinner_rect = egui::Rect::from_center_size(
            egui::pos2(rect.center().x, rect.center().y - 5.0),
            egui::vec2(14.0, 14.0),
        );
        ui.put(spinner_rect, egui::Spinner::new().size(14.0));
    } else {
        let icon_text = if let Some(suffix) = args.suffix {
            format!("{} {}", args.icon, suffix)
        } else {
            args.icon.to_owned()
        };
        // Icon centered in upper portion of button
        ui.painter().text(
            egui::pos2(rect.center().x, rect.center().y - 5.0),
            egui::Align2::CENTER_CENTER,
            icon_text,
            egui::FontId::proportional(16.0),
            text_color,
        );
    }

    // Label centered in lower portion of button
    ui.painter().text(
        egui::pos2(rect.center().x, rect.center().y + 10.0),
        egui::Align2::CENTER_CENTER,
        args.label,
        egui::FontId::proportional(10.0),
        text_color,
    );

    args.enabled && response.clicked()
}

struct ToolbarMenuButtonArgs<'a> {
    width: f32,
    icon: &'a str,
    label: &'a str,
    suffix: Option<&'a str>,
    enabled: bool,
    busy: bool,
}

fn toolbar_menu_button(
    ui: &mut egui::Ui,
    args: ToolbarMenuButtonArgs<'_>,
    add_contents: impl FnOnce(&mut egui::Ui),
) {
    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(args.width, ACTION_HEIGHT),
        if args.enabled {
            egui::Sense::click()
        } else {
            egui::Sense::hover()
        },
    );

    let popup_id = response.id.with("popup");
    let is_open = egui::Popup::is_id_open(ui.ctx(), popup_id);

    if args.enabled && (response.hovered() || is_open) {
        ui.painter()
            .rect_filled(rect, 4.0, egui::Color32::from_white_alpha(18));
    }

    let text_color = if args.enabled {
        ui.visuals().text_color()
    } else {
        ui.visuals().widgets.noninteractive.text_color()
    };

    if args.busy {
        let spinner_rect = egui::Rect::from_center_size(
            egui::pos2(rect.center().x, rect.center().y - 5.0),
            egui::vec2(14.0, 14.0),
        );
        ui.put(spinner_rect, egui::Spinner::new().size(14.0));
    } else {
        let icon_text = if let Some(suffix) = args.suffix {
            format!("{} {}", args.icon, suffix)
        } else {
            args.icon.to_owned()
        };
        // Icon centered in upper portion of button
        ui.painter().text(
            egui::pos2(rect.center().x, rect.center().y - 5.0),
            egui::Align2::CENTER_CENTER,
            icon_text,
            egui::FontId::proportional(16.0),
            text_color,
        );
    }

    // Label centered in lower portion of button
    ui.painter().text(
        egui::pos2(rect.center().x, rect.center().y + 10.0),
        egui::Align2::CENTER_CENTER,
        args.label,
        egui::FontId::proportional(10.0),
        text_color,
    );

    if !args.enabled {
        return;
    }

    egui::Popup::from_toggle_button_response(&response)
        .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
        .show(add_contents);
}
