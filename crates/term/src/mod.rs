use alacritty_terminal::Term;
use alacritty_terminal::event::{Event, EventListener};
use alacritty_terminal::grid::Dimensions;
use alacritty_terminal::term::Config;
use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem};
use std::io::{Read, Write};
use std::sync::mpsc;
use std::thread;

pub struct PtyHandle {
    writer: Box<dyn Write + Send>,
    child: Box<dyn portable_pty::Child + Send>,
}

pub struct TerminalBackend {
    pub term: Term<EventProxy>,
    pub pty: Option<PtyHandle>,
    pub event_rx: mpsc::Receiver<Event>,
    event_tx: mpsc::Sender<Event>,
    parser: alacritty_terminal::vte::ansi::Processor,
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
            parser: alacritty_terminal::vte::ansi::Processor::new(),
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

        self.pty = Some(PtyHandle { writer, child });

        let tx = self.event_tx.clone();

        thread::spawn(move || {
            let mut buf = [0u8; 4096];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        let data = buf[..n].to_vec();
                        let _ =
                            tx.send(Event::PtyWrite(String::from_utf8_lossy(&data).to_string()));
                    }
                    Err(_) => break,
                }
            }
        });
    }

    pub fn feed_pty_output(&mut self, data: &[u8]) {
        self.parser.advance(&mut self.term, data);
    }

    pub fn resize(&mut self, cols: usize, rows: usize) {
        self.term.resize(TermSize { rows, cols });
        if let Some(ref mut pty) = self.pty {
            let _ = pty
                .writer
                .write_all(format!("\x1b[8;{};{}t", rows, cols).as_bytes());
            let _ = pty.writer.flush();
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
}

fn detect_shell() -> String {
    #[cfg(target_os = "windows")]
    {
        std::env::var("COMSPEC").unwrap_or_else(|_| "cmd.exe".to_string())
    }

    #[cfg(not(target_os = "windows"))]
    {
        std::env::var("SHELL").unwrap_or_else(|_| {
            if std::path::Path::new("/bin/zsh").exists() {
                "/bin/zsh".to_string()
            } else if std::path::Path::new("/bin/bash").exists() {
                "/bin/bash".to_string()
            } else {
                "/bin/sh".to_string()
            }
        })
    }
}
