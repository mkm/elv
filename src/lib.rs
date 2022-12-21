#![feature(try_blocks)]
use std::io::Write;
use terminal::{Action, Clear, Value, Retrieved, Event, KeyCode};

mod polyset;
mod pretty;
mod syntax;
mod editor;
mod value;
mod eval;
mod shell;

use shell::Shell;

pub fn run() {
    let mut shell = Shell::new();
    let mut term = terminal::stdout();
    term.act(Action::ClearTerminal(Clear::All)).unwrap();
    term.act(Action::EnableRawMode).unwrap();
    term.act(Action::HideCursor).unwrap();
    loop {
        term.batch(Action::ClearTerminal(Clear::All)).unwrap();
        term.batch(Action::MoveCursorTo(0, 0)).unwrap();
        // write!(&mut term, "{shell:?}").unwrap();
        shell.render(&mut term);
        term.flush_batch().unwrap();
        term.flush().unwrap();
        let event = term.get(Value::Event(None)).unwrap();
        match event {
            Retrieved::Event(Some(Event::Key(ke))) => {
                if ke.code == KeyCode::Esc {
                    break;
                } else {
                    shell.handle_key_event(ke);
                }
            },
            _ =>
                (),
        }
    }
    term.act(Action::ShowCursor).unwrap();
}
