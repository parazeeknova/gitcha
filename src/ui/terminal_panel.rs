use alacritty_terminal::grid::Dimensions;
use eframe::egui;

use crate::term::TerminalBackend;

pub struct State {
    pub open: bool,
    pub height: f32,
    backend: Option<TerminalBackend>,
    needs_repaint: bool,
    focused: bool,
}

impl Default for State {
    fn default() -> Self {
        Self {
            open: false,
            height: 200.0,
            backend: None,
            needs_repaint: false,
            focused: false,
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
        if self.backend.is_none() {
            let cols = 80;
            let rows = 24;
            let mut backend = TerminalBackend::new(cols, rows);
            backend.spawn_shell(working_dir, cols, rows);
            self.backend = Some(backend);
        }
        self.open = true;
    }

    pub fn close(&mut self) {
        self.open = false;
        self.backend = None;
    }
}

pub fn show(ui: &mut egui::Ui, rect: egui::Rect, state: &mut State) {
    if !state.open {
        return;
    }

    let fill = egui::Color32::from_rgb(30, 30, 30);
    let header_fill = egui::Color32::from_rgb(40, 40, 40);
    let border = egui::Color32::from_rgb(60, 60, 60);
    let stroke = egui::Stroke::new(1.0_f32, border);

    let panel_rect = rect;

    ui.painter().rect_filled(panel_rect, 0.0, fill);
    ui.painter()
        .rect_stroke(panel_rect, 0.0, stroke, egui::StrokeKind::Inside);

    // Resize grip at the top
    let resize_grip_height = 6.0;
    let resize_grip_rect = egui::Rect::from_min_max(
        panel_rect.left_top(),
        egui::pos2(panel_rect.right(), panel_rect.top() + resize_grip_height),
    );
    let resize_response = ui.interact(
        resize_grip_rect,
        ui.make_persistent_id("terminal_resize"),
        egui::Sense::drag(),
    );
    if resize_response.dragged() {
        state.height = (state.height - resize_response.drag_delta().y).clamp(100.0, 600.0);
        ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeVertical);
    }
    if resize_response.hovered() || resize_response.dragged() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeVertical);
    }

    // Header
    let header_h = 28.0;
    let header_rect = egui::Rect::from_min_max(
        egui::pos2(panel_rect.left(), panel_rect.top() + resize_grip_height),
        egui::pos2(
            panel_rect.right(),
            panel_rect.top() + resize_grip_height + header_h,
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

    // Content area
    let content_rect = egui::Rect::from_min_max(
        egui::pos2(panel_rect.left(), header_rect.bottom()),
        panel_rect.max,
    );

    if let Some(ref mut backend) = state.backend {
        // Process PTY output
        while let Ok(event) = backend.event_rx.try_recv() {
            if let alacritty_terminal::event::Event::PtyWrite(data) = event {
                backend.feed_pty_output(data.as_bytes());
                state.needs_repaint = true;
            }
        }

        if backend.is_alive() {
            ui.ctx().request_repaint();
        }

        // Render terminal grid
        let font_id = egui::FontId::monospace(13.0);
        let cell_w = 8.0;
        let cell_h = 16.0;
        let padding = 6.0;

        let content_inner = content_rect.shrink2(egui::vec2(padding, padding));

        // Get grid dimensions from Term
        let num_cols = backend.term.columns();
        let num_rows = backend.term.screen_lines();

        // Render cells using renderable_content for correct color support
        let renderable = backend.term.renderable_content();
        for indexed in renderable.display_iter {
            let c = indexed.cell.c;
            let row = indexed.point.line.0 as usize;
            let col = indexed.point.column.0;

            if row >= num_rows || col >= num_cols {
                continue;
            }

            let x = content_inner.left() + col as f32 * cell_w;
            let y = content_inner.top() + row as f32 * cell_h;

            if x + cell_w > content_inner.right() || y + cell_h > content_inner.bottom() {
                continue;
            }

            // Convert alacritty color to egui color
            let fg_color = color_to_egui(indexed.cell.fg);

            if c != ' ' {
                ui.painter().text(
                    egui::pos2(x, y + cell_h * 0.8),
                    egui::Align2::LEFT_CENTER,
                    c.to_string(),
                    font_id.clone(),
                    fg_color,
                );
            }
        }

        // Input handling - use a wide interaction area covering the whole content
        let input_rect = content_rect;
        let input_resp = ui.interact(
            input_rect,
            ui.make_persistent_id("terminal_content"),
            egui::Sense::click(),
        );

        if input_resp.clicked() {
            state.focused = true;
            ui.ctx().memory_mut(|m| m.request_focus(input_resp.id));
        }

        if state.focused && ui.memory(|m| m.has_focus(input_resp.id)) {
            ui.ctx().input_mut(|i| {
                for event in &i.events.clone() {
                    match event {
                        egui::Event::Text(text) => {
                            // Only send printable characters, not control sequences
                            if !text.chars().all(|c| c.is_control()) {
                                backend.write_input(text.as_bytes());
                                state.needs_repaint = true;
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
                                state.needs_repaint = true;
                            }
                        }
                        _ => {}
                    }
                }
            });
        } else if state.focused && !ui.memory(|m| m.has_focus(input_resp.id)) {
            state.focused = false;
        }

        if state.needs_repaint {
            ui.ctx().request_repaint();
            state.needs_repaint = false;
        }

        // Draw cursor when focused
        if state.focused {
            let cursor_blink = (ui.ctx().input(|i| i.time) * 2.0).sin() > 0.0;
            if cursor_blink {
                // Get cursor position from term
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
    use alacritty_terminal::vte::ansi::Color;

    match color {
        Color::Named(name) => {
            // Simple named color mapping
            let idx = name as u8;
            match idx {
                0 => egui::Color32::from_rgb(0, 0, 0),
                1 => egui::Color32::from_rgb(205, 49, 49),
                2 => egui::Color32::from_rgb(13, 188, 121),
                3 => egui::Color32::from_rgb(229, 229, 16),
                4 => egui::Color32::from_rgb(36, 114, 200),
                5 => egui::Color32::from_rgb(188, 63, 188),
                6 => egui::Color32::from_rgb(17, 168, 205),
                7 => egui::Color32::from_rgb(229, 229, 229),
                8 => egui::Color32::from_rgb(102, 102, 102),
                9 => egui::Color32::from_rgb(241, 76, 76),
                10 => egui::Color32::from_rgb(35, 209, 139),
                11 => egui::Color32::from_rgb(245, 245, 67),
                12 => egui::Color32::from_rgb(59, 142, 234),
                13 => egui::Color32::from_rgb(214, 112, 214),
                14 => egui::Color32::from_rgb(41, 184, 219),
                15 => egui::Color32::from_rgb(229, 229, 229),
                _ => egui::Color32::from_rgb(200, 200, 200),
            }
        }
        Color::Spec(rgb) => egui::Color32::from_rgb(rgb.r, rgb.g, rgb.b),
        Color::Indexed(idx) => {
            if idx < 16 {
                let colors = [
                    (0, 0, 0),
                    (205, 49, 49),
                    (13, 188, 121),
                    (229, 229, 16),
                    (36, 114, 200),
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
