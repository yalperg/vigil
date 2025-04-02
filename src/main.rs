use anyhow::Ok;
use crossterm::{cursor, event::{self, read}, style, terminal, ExecutableCommand, QueueableCommand};
use std::io::{stdout, Write};

enum Action {
    Quit,

    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,

    EnterMode(Mode),
}

enum Mode {
    Normal,
    Insert,
}

fn handle_event(mode: &Mode, stdout: &mut std::io::Stdout, ev: event::Event) -> anyhow::Result<Option<Action>> {
    match mode {
        Mode::Normal => handle_normal_event(ev),
        Mode::Insert => handle_insert_event(stdout, ev)
    }
}

fn handle_normal_event(ev: event::Event) -> anyhow::Result<Option<Action>> {
    match ev {
        event::Event::Key(key_event) => {
            match key_event.code {
                event::KeyCode::Esc => Ok(Some(Action::Quit)),
                event::KeyCode::Up | event::KeyCode::Char('k') => Ok(Some(Action::MoveUp)),
                event::KeyCode::Down | event::KeyCode::Char('j') => Ok(Some(Action::MoveDown)),
                event::KeyCode::Left | event::KeyCode::Char('h') => Ok(Some(Action::MoveLeft)),
                event::KeyCode::Right | event::KeyCode::Char('l') => Ok(Some(Action::MoveRight)),
                event::KeyCode::Char('i') => Ok(Some(Action::EnterMode(Mode::Insert))),
                _ => Ok(None),
            }
        },
        _ => Ok(None),
    }
}

fn handle_insert_event(stdout: &mut std::io::Stdout, ev: event::Event) -> anyhow::Result<Option<Action>> {
    match ev {
        event::Event::Key(key_event) => {
            match key_event.code {
                event::KeyCode::Esc => Ok(Some(Action::EnterMode(Mode::Normal))),
                event::KeyCode::Char(c) => {
                    stdout.queue(style::Print(c))?;
                    Ok(None)
                },
                _ => Ok(None),
            }
        },
        _ => Ok(None),
    }
}

fn main() -> anyhow::Result<()> {
    let mut stdout = stdout();
    let mut mode = Mode::Normal;

    let mut cx = 0;
    let mut cy = 0;

    terminal::enable_raw_mode().unwrap();
    stdout.execute(terminal::EnterAlternateScreen)?;

    stdout.execute(terminal::Clear(terminal::ClearType::All))?;

    loop {
        stdout.queue(cursor::MoveTo(cx, cy))?;
        stdout.flush()?;
        
        if let Some(action) = handle_event(&mode, &mut stdout, read()?)? {
            match action {
                Action::Quit => break,
                Action::MoveUp => {
                    cy = cy.saturating_sub(1);
                },
                Action::MoveDown => {
                    cy += 1;
                },
                Action::MoveLeft => {
                    cx = cx.saturating_sub(1);
                },
                Action::MoveRight => {
                    cx += 1;
                },
                Action::EnterMode(new_mode) => {
                    mode = new_mode;
                },
            }
        };
    }

    stdout.execute(terminal::LeaveAlternateScreen)?;
    terminal::disable_raw_mode().unwrap();

    Ok(())
}
