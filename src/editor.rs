use crossterm::{
    cursor,
    event::{self, read},
    style::{self, Stylize},
    terminal, ExecutableCommand, QueueableCommand,
};
use std::io::{stdout, Write};

use crate::buffer::Buffer;

enum Action {
    Quit,

    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,

    AddChar(char),
    DeleteChar,
    NewLine,

    EnterMode(Mode),
}

#[derive(Debug)]
enum Mode {
    Normal,
    Insert,
}

pub struct Editor {
    buffer: Buffer,
    stdout: std::io::Stdout,
    size: (u16, u16),
    mode: Mode,
    cx: u16,
    cy: u16,
}

impl Drop for Editor {
    fn drop(&mut self) {
        _ = self.stdout.flush();
        _ = self.stdout.execute(terminal::LeaveAlternateScreen);
        _ = terminal::disable_raw_mode();
    }
}

impl Editor {
    pub fn new(buffer: Buffer) -> anyhow::Result<Self> {
        let mut stdout = stdout();

        terminal::enable_raw_mode().unwrap();
        stdout
            .execute(terminal::EnterAlternateScreen)?
            .execute(terminal::Clear(terminal::ClearType::All))?;

        Ok(Editor {
            buffer,
            stdout,
            size: terminal::size().unwrap(),
            mode: Mode::Normal,
            cx: 0,
            cy: 0,
        })
    }

    fn draw(&mut self) -> anyhow::Result<()> {
        self.draw_buffer()?;
        self.draw_statusline()?;
        self.stdout.queue(cursor::MoveTo(self.cx, self.cy))?;
        self.stdout.flush()?;
        Ok(())
    }

    fn draw_buffer(&mut self) -> anyhow::Result<()> {
        for (i, line) in self.buffer.lines.iter().enumerate() {
            if i >= self.size.1 as usize - 2 {
                break;
            }
            self.stdout.queue(cursor::MoveTo(0, i as u16))?;
            self.stdout.queue(style::Print(line))?;
        }
        Ok(())
    }

    fn draw_statusline(&mut self) -> anyhow::Result<()> {
        let mode = format!(" {:?} ", self.mode).to_uppercase();
        let file = format!(" {}", self.buffer.file.as_deref().unwrap_or("No Name"));
        let pos = format!(" {}:{} ", self.cx, self.cy);

        let file_width = self.size.0 - mode.len() as u16 - pos.len() as u16 - 2;

        self.stdout.queue(cursor::MoveTo(0, self.size.1 - 2))?;
        self.stdout.queue(style::PrintStyledContent(
            mode.with(style::Color::Rgb { r: 0, g: 0, b: 0 })
                .on(style::Color::Rgb {
                    r: 184,
                    g: 144,
                    b: 243,
                }),
        ))?;
        self.stdout.queue(style::PrintStyledContent(
            ""
                .with(style::Color::Rgb {
                    r: 184,
                    g: 144,
                    b: 243,
                })
                .on(style::Color::Rgb {
                    r: 67,
                    g: 70,
                    b: 89,
                }),
        ))?;
        self.stdout.queue(style::PrintStyledContent(
            format!("{:<width$}", file, width = file_width as usize)
                .with(style::Color::Rgb {
                    r: 255,
                    g: 255,
                    b: 255,
                })
                .on(style::Color::Rgb {
                    r: 67,
                    g: 70,
                    b: 89,
                }),
        ))?;
        self.stdout.queue(style::PrintStyledContent(
            ""
                .with(style::Color::Rgb {
                    r: 184,
                    g: 144,
                    b: 243,
                })
                .on(style::Color::Rgb {
                    r: 67,
                    g: 70,
                    b: 89,
                }),
        ))?;
        self.stdout.queue(style::PrintStyledContent(
            pos.with(style::Color::Rgb { r: 0, g: 0, b: 0 })
                .bold()
                .on(style::Color::Rgb {
                    r: 184,
                    g: 144,
                    b: 243,
                }),
        ))?;

        Ok(())
    }

    pub fn run(&mut self) -> anyhow::Result<()> {
        loop {
            self.draw()?;

            if let Some(action) = self.handle_event(read()?)? {
                match action {
                    Action::Quit => break,
                    Action::MoveUp => {
                        self.cy = self.cy.saturating_sub(1);
                    }
                    Action::MoveDown => {
                        self.cy += 1;
                    }
                    Action::MoveLeft => {
                        self.cx = self.cx.saturating_sub(1);
                    }
                    Action::MoveRight => {
                        self.cx += 1;
                    }
                    Action::EnterMode(new_mode) => {
                        self.mode = new_mode;
                    }
                    Action::AddChar(c) => {
                        self.stdout.queue(cursor::MoveTo(self.cx, self.cy))?;
                        self.stdout.queue(style::Print(c))?;
                        self.cx += 1;
                    }
                    Action::DeleteChar => {
                        if self.cx > 0 {
                            self.cx -= 1;
                        } else {
                            self.cy = self.cy.saturating_sub(1);
                            self.cx = terminal::size()?.0.saturating_sub(1);
                        }

                        self.stdout.queue(cursor::MoveTo(self.cx, self.cy))?;
                        self.stdout.queue(style::Print(' '))?;
                        self.stdout.queue(cursor::MoveTo(self.cx, self.cy))?;
                    }
                    Action::NewLine => {
                        self.cy += 1;
                        self.cx = 0;
                    }
                }
            };
        }

        Ok(())
    }

    fn handle_event(&mut self, ev: event::Event) -> anyhow::Result<Option<Action>> {
        if matches!(ev, event::Event::Resize(_, _)) {
            self.size = terminal::size()?;
        }
        match self.mode {
            Mode::Normal => self.handle_normal_event(ev),
            Mode::Insert => self.handle_insert_event(ev),
        }
    }

    fn handle_normal_event(&self, ev: event::Event) -> anyhow::Result<Option<Action>> {
        let action = match ev {
            event::Event::Key(event) => match event.code {
                event::KeyCode::Esc | event::KeyCode::Char('q') => Some(Action::Quit),
                event::KeyCode::Up | event::KeyCode::Char('k') => Some(Action::MoveUp),
                event::KeyCode::Down | event::KeyCode::Char('j') => Some(Action::MoveDown),
                event::KeyCode::Left | event::KeyCode::Char('h') => Some(Action::MoveLeft),
                event::KeyCode::Right | event::KeyCode::Char('l') => Some(Action::MoveRight),
                event::KeyCode::Char('i') => Some(Action::EnterMode(Mode::Insert)),
                _ => None,
            },
            _ => None,
        };

        Ok(action)
    }

    fn handle_insert_event(&self, ev: event::Event) -> anyhow::Result<Option<Action>> {
        let action = match ev {
            event::Event::Key(event) => match event.code {
                event::KeyCode::Esc => Some(Action::EnterMode(Mode::Normal)),
                event::KeyCode::Char(c) => Some(Action::AddChar(c)),
                event::KeyCode::Backspace => Some(Action::DeleteChar),
                event::KeyCode::Enter => Some(Action::NewLine),
                _ => None,
            },
            _ => None,
        };

        Ok(action)
    }
}
