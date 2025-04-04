use std::{io::stdout, panic};

use crossterm::{terminal, ExecutableCommand};
use editor::Editor;
use buffer::Buffer;

mod logger;
mod editor;
mod buffer;

fn main() -> anyhow::Result<()> {
    let file = std::env::args().nth(1);
    let buffer = Buffer::from_file(file);
    let mut editor = Editor::new(buffer)?;

    panic::set_hook(Box::new(|info| {
        _ = stdout().execute(terminal::LeaveAlternateScreen);
        _ = terminal::disable_raw_mode();

        eprintln!("Error: {}", info);
    }));

    editor.run()?;
    editor.cleanup()
}
