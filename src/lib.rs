#![feature(try_blocks)]
#![feature(iter_intersperse)]
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

struct Cleanup {}

impl Drop for Cleanup {
    fn drop(&mut self) {
        let term = terminal::stdout();
        term.act(Action::ClearTerminal(Clear::All)).unwrap();
        term.act(Action::MoveCursorTo(0, 0)).unwrap();
        term.act(Action::ShowCursor).unwrap();
        term.act(Action::DisableRawMode).unwrap();
    }
}

pub fn run() {
    let mut shell = Shell::new();
    let mut term = terminal::stdout();
    term.act(Action::ClearTerminal(Clear::All)).unwrap();
    term.act(Action::EnableRawMode).unwrap();
    term.act(Action::HideCursor).unwrap();
    let _cleanup = Cleanup {};
    loop {
        term.batch(Action::ClearTerminal(Clear::All)).unwrap();
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
}
