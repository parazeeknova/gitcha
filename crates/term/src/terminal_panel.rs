use alacritty_terminal::grid::Dimensions;
use eframe::egui;

use crate::term::TerminalBackend;

const PADDING: f32 = 6.0;
const HEADER_H: f32 = 28.0;
const RESIZE_GRIP_H: f32 = 6.0;
const MIN_HEIGHT: f32 = 100.0;
const MAX_HEIGHT: f32 = 600.0;

pub struct State {
    pub open: bool,
    pub height: f32,
    backend: Option<TerminalBackend>,
    last_cols: usize,
    last_rows: usize,
    focused: bool,
    pending_spawn: Option<String>,
    cell_w: f32,
    cell_h: f32,
}

impl Default for State {
    fn default() -> Self {
        Self {
            open: false,
            height: 200.0,
            backend: None,
            last_cols: 1,
            last_rows: 1,
            focused: false,
            pending_spawn: None,
            cell_w: 8.0,
            cell_h: 16.0,
        }
    }
}

impl State {
    pub fn toggle(&mut self, working_dir: &str) {
        if self.open {
            self.close();
        } else {
            self.open(working_dir);
        }
    }

    pub fn open(&mut self, working_dir: &str) {
        self.open = true;
        if self.backend.is_none() {
            self.pending_spawn = Some(working_dir.to_string());
        }
    }

    pub fn is_focused(&self) -> bool {
        self.focused && self.open
    }

    pub fn has_pending_spawn(&self) -> bool {
        self.pending_spawn.is_some()
    }

    pub fn is_initialized(&self) -> bool {
        self.backend.is_some()
    }

    pub fn send_command(&mut self, cmd: &str) {
        if let Some(ref mut backend) = self.backend {
            let mut input = cmd.as_bytes().to_vec();
            input.push(b'\r');
            backend.write_input(&input);
        }
    }

    pub fn close(&mut self) {
        self.open = false;
        self.pending_spawn = None;
        if let Some(mut backend) = self.backend.take() {
            backend.close();
        }
    }
}

fn measure_cell(ui: &egui::Ui) -> (f32, f32) {
    let font_id = egui::FontId::monospace(13.0);
    let galley = ui
        .painter()
        .layout_no_wrap("M".to_string(), font_id, egui::Color32::WHITE);
    let w = galley.size().x;
    let h = galley.size().y.max(16.0);
    (w, h)
}

pub fn show(ui: &mut egui::Ui, rect: egui::Rect, state: &mut State) {
    show_inner(ui, rect, state, false);
}

pub fn show_headerless(ui: &mut egui::Ui, rect: egui::Rect, state: &mut State) {
    show_inner(ui, rect, state, true);
}

fn show_inner(ui: &mut egui::Ui, rect: egui::Rect, state: &mut State, headerless: bool) {
    if !state.open {
        return;
    }

    let (measured_w, measured_h) = measure_cell(ui);
    state.cell_w = measured_w;
    state.cell_h = measured_h;

    let fill = egui::Color32::from_rgb(30, 30, 30);
    let border = egui::Color32::from_rgb(60, 60, 60);
    let stroke = egui::Stroke::new(1.0_f32, border);

    let panel_rect = rect;

    ui.painter().rect_filled(panel_rect, 0.0, fill);
    ui.painter()
        .rect_stroke(panel_rect, 0.0, stroke, egui::StrokeKind::Inside);

    let content_rect = if headerless {
        panel_rect
    } else {
        let resize_grip_rect = egui::Rect::from_min_max(
            panel_rect.left_top(),
            egui::pos2(panel_rect.right(), panel_rect.top() + RESIZE_GRIP_H),
        );
        let resize_response = ui.interact(
            resize_grip_rect,
            ui.make_persistent_id("terminal_resize"),
            egui::Sense::drag(),
        );
        if resize_response.dragged() {
            state.height =
                (state.height - resize_response.drag_delta().y).clamp(MIN_HEIGHT, MAX_HEIGHT);
            ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeVertical);
        }
        if resize_response.hovered() || resize_response.dragged() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeVertical);
        }

        let header_fill = egui::Color32::from_rgb(40, 40, 40);
        let header_rect = egui::Rect::from_min_max(
            egui::pos2(panel_rect.left(), panel_rect.top() + RESIZE_GRIP_H),
            egui::pos2(
                panel_rect.right(),
                panel_rect.top() + RESIZE_GRIP_H + HEADER_H,
            ),
        );

        ui.painter().rect_filled(header_rect, 0.0, header_fill);
        ui.painter().line_segment(
            [header_rect.left_bottom(), header_rect.right_bottom()],
            stroke,
        );

        ui.painter().text(
            egui::pos2(header_rect.left() + 12.0, header_rect.center().y),
            egui::Align2::LEFT_CENTER,
            "Terminal",
            egui::FontId::proportional(11.0),
            egui::Color32::from_rgb(180, 180, 180),
        );

        let close_btn_rect = egui::Rect::from_center_size(
            egui::pos2(header_rect.right() - 14.0, header_rect.center().y),
            egui::vec2(20.0, 20.0),
        );
        let close_resp = ui.interact(
            close_btn_rect,
            ui.make_persistent_id("terminal_close"),
            egui::Sense::click(),
        );
        if close_resp.hovered() {
            ui.painter()
                .rect_filled(close_btn_rect, 3.0, egui::Color32::from_rgb(80, 80, 80));
        }
        ui.painter().text(
            close_btn_rect.center(),
            egui::Align2::CENTER_CENTER,
            "\u{00d7}",
            egui::FontId::proportional(14.0),
            egui::Color32::from_rgb(180, 180, 180),
        );
        if close_resp.clicked() {
            state.close();
            return;
        }

        egui::Rect::from_min_max(
            egui::pos2(panel_rect.left(), header_rect.bottom()),
            panel_rect.max,
        )
    };

    let content_inner = content_rect.shrink2(egui::vec2(PADDING, PADDING));

    let cell_w = state.cell_w;
    let cell_h = state.cell_h;

    if content_inner.width() < cell_w || content_inner.height() < cell_h {
        ui.painter().text(
            content_rect.center(),
            egui::Align2::CENTER_CENTER,
            "Terminal too small",
            egui::FontId::proportional(12.0),
            egui::Color32::from_rgb(120, 120, 120),
        );
        return;
    }

    let computed_cols = ((content_inner.width() / cell_w).floor() as usize).max(1);
    let computed_rows = ((content_inner.height() / cell_h).floor() as usize).max(1);

    if let Some(working_dir) = state.pending_spawn.take() {
        let mut backend = TerminalBackend::new(computed_cols, computed_rows);
        backend.spawn_shell(&working_dir, computed_cols, computed_rows);
        state.backend = Some(backend);
        state.last_cols = computed_cols;
        state.last_rows = computed_rows;
        ui.ctx().request_repaint();
        return;
    }

    if let Some(ref mut backend) = state.backend {
        let mut has_new_data = false;

        while let Ok(data) = backend.pty_rx.try_recv() {
            backend.feed_pty_output(&data);
            has_new_data = true;
        }

        if has_new_data {
            ui.ctx().request_repaint();
        }

        if computed_cols != state.last_cols || computed_rows != state.last_rows {
            backend.resize(computed_cols, computed_rows);
            state.last_cols = computed_cols;
            state.last_rows = computed_rows;
        }

        let font_id = egui::FontId::monospace(13.0);

        let num_cols = backend.term.columns();
        let num_rows = backend.term.screen_lines();

        let renderable = backend.term.renderable_content();

        let mut row_cells: Vec<Vec<(char, egui::Color32)>> =
            vec![vec![(' ', egui::Color32::WHITE); num_cols]; num_rows];

        for indexed in renderable.display_iter {
            let row = indexed.point.line.0 as usize;
            let col = indexed.point.column.0;
            if row < num_rows && col < num_cols {
                row_cells[row][col] = (indexed.cell.c, color_to_egui(indexed.cell.fg));
            }
        }

        for (row, cells) in row_cells.iter().enumerate().take(num_rows) {
            let y = content_inner.top() + row as f32 * cell_h;
            let mut x_offset = content_inner.left();

            let mut seg_start = 0;
            while seg_start < cells.len() {
                let mut seg_end = seg_start + 1;
                let seg_color = cells[seg_start].1;
                while seg_end < cells.len() && cells[seg_end].1 == seg_color {
                    seg_end += 1;
                }
                let text: String = cells[seg_start..seg_end].iter().map(|(c, _)| *c).collect();
                let galley = ui
                    .painter()
                    .layout_no_wrap(text, font_id.clone(), seg_color);
                let width = galley.size().x;
                ui.painter()
                    .galley(egui::pos2(x_offset, y), galley, seg_color);
                x_offset += width;
                seg_start = seg_end;
            }
        }

        let input_rect = content_rect;
        let input_resp = ui.interact(
            input_rect,
            ui.make_persistent_id("terminal_content"),
            egui::Sense::click(),
        );

        if input_resp.hovered() || state.focused {
            ui.ctx().set_cursor_icon(egui::CursorIcon::Text);
        }

        if input_resp.clicked() {
            state.focused = true;
            ui.ctx().memory_mut(|m| m.request_focus(input_resp.id));
        }

        if state.focused && ui.memory(|m| m.has_focus(input_resp.id)) {
            let mut needs_repaint = false;
            ui.ctx().input_mut(|i| {
                for event in &i.events.clone() {
                    match event {
                        egui::Event::Text(text) => {
                            if !text.chars().all(|c| c.is_control()) {
                                backend.write_input(text.as_bytes());
                                needs_repaint = true;
                            }
                        }
                        egui::Event::Key {
                            key,
                            pressed: true,
                            modifiers,
                            ..
                        } => {
                            if let Some(bytes) = key_to_bytes(*key, *modifiers) {
                                backend.write_input(&bytes);
                                needs_repaint = true;
                            }
                        }
                        _ => {}
                    }
                }
            });
            if needs_repaint {
                ui.ctx().request_repaint();
            }
        } else if state.focused && !ui.memory(|m| m.has_focus(input_resp.id)) {
            state.focused = false;
        }

        if state.focused {
            let cursor_blink = (ui.ctx().input(|i| i.time) * 2.0).sin() > 0.0;
            ui.ctx()
                .request_repaint_after(std::time::Duration::from_millis(500));
            if cursor_blink {
                let cursor_point = backend.term.grid().cursor.point;
                let cursor_row = cursor_point.line.0 as usize;
                let cursor_col = cursor_point.column.0;

                if cursor_row < num_rows && cursor_col < num_cols {
                    let cx = content_inner.left() + cursor_col as f32 * cell_w;
                    let cy = content_inner.top() + cursor_row as f32 * cell_h;
                    ui.painter().rect_filled(
                        egui::Rect::from_min_size(
                            egui::pos2(cx, cy + 2.0),
                            egui::vec2(cell_w, cell_h - 4.0),
                        ),
                        1.0,
                        egui::Color32::from_rgb(200, 200, 200),
                    );
                }
            }
        }
    } else {
        ui.painter().text(
            content_rect.center(),
            egui::Align2::CENTER_CENTER,
            "Terminal not initialized",
            egui::FontId::proportional(12.0),
            egui::Color32::from_rgb(120, 120, 120),
        );
    }
}

fn color_to_egui(color: alacritty_terminal::vte::ansi::Color) -> egui::Color32 {
    use alacritty_terminal::vte::ansi::{Color, NamedColor};

    match color {
        Color::Named(name) => match name {
            NamedColor::Black => egui::Color32::from_rgb(1, 1, 1),
            NamedColor::Red => egui::Color32::from_rgb(205, 49, 49),
            NamedColor::Green => egui::Color32::from_rgb(13, 188, 121),
            NamedColor::Yellow => egui::Color32::from_rgb(229, 229, 16),
            NamedColor::Blue => egui::Color32::from_rgb(59, 142, 234),
            NamedColor::Magenta => egui::Color32::from_rgb(188, 63, 188),
            NamedColor::Cyan => egui::Color32::from_rgb(17, 168, 205),
            NamedColor::White => egui::Color32::from_rgb(229, 229, 229),
            NamedColor::BrightBlack => egui::Color32::from_rgb(102, 102, 102),
            NamedColor::BrightRed => egui::Color32::from_rgb(241, 76, 76),
            NamedColor::BrightGreen => egui::Color32::from_rgb(35, 209, 139),
            NamedColor::BrightYellow => egui::Color32::from_rgb(245, 245, 67),
            NamedColor::BrightBlue => egui::Color32::from_rgb(59, 142, 234),
            NamedColor::BrightMagenta => egui::Color32::from_rgb(214, 112, 214),
            NamedColor::BrightCyan => egui::Color32::from_rgb(41, 184, 219),
            NamedColor::BrightWhite => egui::Color32::from_rgb(229, 229, 229),
            NamedColor::Foreground => egui::Color32::from_rgb(229, 229, 229),
            NamedColor::Background => egui::Color32::from_rgb(30, 30, 30),
            _ => egui::Color32::from_rgb(229, 229, 229),
        },
        Color::Spec(rgb) => egui::Color32::from_rgb(rgb.r, rgb.g, rgb.b),
        Color::Indexed(idx) => {
            if idx < 16 {
                let colors = [
                    (1, 1, 1),
                    (205, 49, 49),
                    (13, 188, 121),
                    (229, 229, 16),
                    (59, 142, 234),
                    (188, 63, 188),
                    (17, 168, 205),
                    (229, 229, 229),
                    (102, 102, 102),
                    (241, 76, 76),
                    (35, 209, 139),
                    (245, 245, 67),
                    (59, 142, 234),
                    (214, 112, 214),
                    (41, 184, 219),
                    (229, 229, 229),
                ];
                let (r, g, b) = colors[idx as usize];
                egui::Color32::from_rgb(r, g, b)
            } else if idx < 232 {
                let idx = idx - 16;
                let r = (idx / 36) * 51;
                let g = ((idx % 36) / 6) * 51;
                let b = (idx % 6) * 51;
                egui::Color32::from_rgb(r, g, b)
            } else {
                let gray = (idx - 232) * 10 + 8;
                egui::Color32::from_rgb(gray, gray, gray)
            }
        }
    }
}

fn key_to_bytes(key: egui::Key, modifiers: egui::Modifiers) -> Option<Vec<u8>> {
    match key {
        egui::Key::Enter => Some(b"\r".to_vec()),
        egui::Key::Backspace => Some(b"\x7f".to_vec()),
        egui::Key::Delete => Some(b"\x1b[3~".to_vec()),
        egui::Key::ArrowUp => Some(b"\x1b[A".to_vec()),
        egui::Key::ArrowDown => Some(b"\x1b[B".to_vec()),
        egui::Key::ArrowRight => Some(b"\x1b[C".to_vec()),
        egui::Key::ArrowLeft => Some(b"\x1b[D".to_vec()),
        egui::Key::Home => Some(b"\x1b[H".to_vec()),
        egui::Key::End => Some(b"\x1b[F".to_vec()),
        egui::Key::PageUp => Some(b"\x1b[5~".to_vec()),
        egui::Key::PageDown => Some(b"\x1b[6~".to_vec()),
        egui::Key::Tab => Some(b"\t".to_vec()),
        egui::Key::Escape => Some(b"\x1b".to_vec()),
        egui::Key::A if modifiers.ctrl => Some(b"\x01".to_vec()),
        egui::Key::C if modifiers.ctrl => Some(b"\x03".to_vec()),
        egui::Key::D if modifiers.ctrl => Some(b"\x04".to_vec()),
        egui::Key::Z if modifiers.ctrl => Some(b"\x1a".to_vec()),
        _ => None,
    }
}
