use alacritty_terminal::Term;
use alacritty_terminal::event::{Event, EventListener};
use alacritty_terminal::grid::Dimensions;
use alacritty_terminal::term::Config;
use portable_pty::{CommandBuilder, MasterPty, NativePtySystem, PtySize, PtySystem};
use std::io::{Read, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, mpsc};
use std::thread::{self, JoinHandle};

pub mod terminal_panel;

pub struct PtyHandle {
    writer: Box<dyn Write + Send>,
    master: Box<dyn MasterPty + Send>,
    child: Box<dyn portable_pty::Child + Send>,
}

pub struct TerminalBackend {
    pub term: Term<EventProxy>,
    pub pty: Option<PtyHandle>,
    pub event_rx: mpsc::Receiver<Event>,
    #[allow(dead_code)]
    event_tx: mpsc::Sender<Event>,
    pub pty_rx: mpsc::Receiver<Vec<u8>>,
    pty_tx: mpsc::Sender<Vec<u8>>,
    parser: alacritty_terminal::vte::ansi::Processor,
    stop_flag: Arc<AtomicBool>,
    reader_handle: Option<JoinHandle<()>>,
}

pub struct EventProxy {
    tx: mpsc::Sender<Event>,
}

impl EventListener for EventProxy {
    fn send_event(&self, event: Event) {
        let _ = self.tx.send(event);
    }
}

#[derive(Clone, Copy)]
pub struct TermSize {
    pub rows: usize,
    pub cols: usize,
}

impl Dimensions for TermSize {
    fn total_lines(&self) -> usize {
        self.rows
    }

    fn screen_lines(&self) -> usize {
        self.rows
    }

    fn columns(&self) -> usize {
        self.cols
    }
}

impl TerminalBackend {
    pub fn new(cols: usize, rows: usize) -> Self {
        let (event_tx, event_rx) = mpsc::channel();
        let (pty_tx, pty_rx) = mpsc::channel();

        let size = TermSize { rows, cols };
        let config = Config::default();

        let term = Term::new(
            config,
            &size,
            EventProxy {
                tx: event_tx.clone(),
            },
        );

        Self {
            term,
            pty: None,
            event_rx,
            event_tx,
            pty_rx,
            pty_tx,
            parser: alacritty_terminal::vte::ansi::Processor::new(),
            stop_flag: Arc::new(AtomicBool::new(false)),
            reader_handle: None,
        }
    }

    pub fn spawn_shell(&mut self, working_dir: &str, cols: usize, rows: usize) {
        let pty_system = NativePtySystem::default();

        let pair = pty_system
            .openpty(PtySize {
                rows: rows as u16,
                cols: cols as u16,
                pixel_width: 0,
                pixel_height: 0,
            })
            .expect("Failed to open PTY");

        let shell = detect_shell();
        let mut cmd = CommandBuilder::new(&shell);
        cmd.cwd(working_dir);

        let child = pair
            .slave
            .spawn_command(cmd)
            .expect("Failed to spawn shell");

        let writer = pair.master.take_writer().expect("Failed to get PTY writer");

        let mut reader = pair
            .master
            .try_clone_reader()
            .expect("Failed to get PTY reader");

        let master = pair.master;

        let stop_flag_clone = self.stop_flag.clone();

        let pty_tx = self.pty_tx.clone();
        let handle = thread::spawn(move || {
            let mut buf = [0u8; 4096];
            loop {
                if stop_flag_clone.load(Ordering::Relaxed) {
                    break;
                }
                match reader.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        let data = buf[..n].to_vec();
                        if pty_tx.send(data).is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        self.reader_handle = Some(handle);
        self.pty = Some(PtyHandle {
            writer,
            master,
            child,
        });
    }

    pub fn feed_pty_output(&mut self, data: &[u8]) {
        if let Some(ref mut pty) = self.pty {
            let mut i = 0;
            while i < data.len() {
                if data[i] == 0x1b && i + 2 < data.len() && data[i + 1] == b'[' {
                    let param_start = i + 2;
                    let mut j = param_start;
                    while j < data.len() && (data[j].is_ascii_digit() || data[j] == b';') {
                        j += 1;
                    }
                    if j < data.len()
                        && data[j] == b'>'
                        && j + 1 < data.len()
                        && data[j + 1] == b'c'
                    {
                        let _ = pty.writer.write_all(b"\x1b[?1;2c");
                        let _ = pty.writer.flush();
                        i = j + 2;
                        continue;
                    }
                    if j < data.len() && data[j] == b'c' && j == param_start {
                        let _ = pty.writer.write_all(b"\x1b[?1;2c");
                        let _ = pty.writer.flush();
                        i = j + 1;
                        continue;
                    }
                }
                i += 1;
            }
        }
        self.parser.advance(&mut self.term, data);
    }

    pub fn resize(&mut self, cols: usize, rows: usize) {
        self.term.resize(TermSize { rows, cols });
        if let Some(ref mut pty) = self.pty {
            let _ = pty.master.resize(PtySize {
                rows: rows as u16,
                cols: cols as u16,
                pixel_width: 0,
                pixel_height: 0,
            });
        }
    }

    pub fn write_input(&mut self, data: &[u8]) {
        if let Some(ref mut pty) = self.pty {
            let _ = pty.writer.write_all(data);
            let _ = pty.writer.flush();
        }
    }

    pub fn is_alive(&mut self) -> bool {
        self.pty
            .as_mut()
            .is_some_and(|p| p.child.try_wait().ok().flatten().is_none())
    }

    pub fn close(&mut self) {
        self.stop_flag.store(true, Ordering::Relaxed);

        if let Some(ref mut pty) = self.pty {
            let _ = pty.child.kill();
        }

        if let Some(handle) = self.reader_handle.take() {
            let _ = handle.join();
        }

        if let Some(pty) = self.pty.take() {
            drop(pty.writer);
            drop(pty.master);
            drop(pty.child);
        }
    }
}

impl Drop for TerminalBackend {
    fn drop(&mut self) {
        self.close();
    }
}

fn detect_shell() -> String {
    #[cfg(target_os = "windows")]
    {
        std::env::var("COMSPEC").unwrap_or_else(|_| "cmd.exe".to_string())
    }

    #[cfg(not(target_os = "windows"))]
    {
        if std::path::Path::new("/bin/bash").exists() {
            "/bin/bash".to_string()
        } else {
            "/bin/sh".to_string()
        }
    }
}
