use std::collections::HashMap;
use std::io::Write;
use terminal::{Terminal, KeyEvent, KeyCode, Value, Retrieved, Color};
use crate::{
    editor::{Cursor, Mode},
    pretty::{Pretty, Pos, Size, Layout, Symbol},
    eval::VM,
};

#[derive(Debug, Clone)]
pub struct Shell {
    cursor: Cursor,
}

impl Shell {
    pub fn new() -> Self {
        Self {
            cursor: Cursor::empty(),
        }
    }

    pub fn handle_key_event(&mut self, event: KeyEvent) {
        match self.cursor.mode() {
            Mode::Normal => self.handle_key_event_normal(event),
            Mode::Ident => self.handle_key_event_ident(event),
            Mode::StrLit => self.handle_key_event_strlit(event),
        }
    }

    pub fn handle_key_event_normal(&mut self, event: KeyEvent) {
        if event.modifiers.is_empty() {
            match event.code {
                KeyCode::Left =>
                    self.cursor.move_left(),
                KeyCode::Right =>
                    self.cursor.move_right(),
                KeyCode::Up =>
                    self.cursor.move_up(),
                KeyCode::Backspace =>
                    self.cursor.delete_before(),
                KeyCode::Char('i') =>
                    self.cursor.insert(Cursor::empty_ident()),
                KeyCode::Char('"') =>
                    self.cursor.insert(Cursor::empty_str_lit()),
                KeyCode::Char('{') =>
                    self.cursor.insert(Cursor::empty_quote()),
                KeyCode::Char('}') | KeyCode::Down =>
                    self.cursor.move_out(),
                _ =>
                    (),
            }
        }
    }

    pub fn handle_key_event_ident(&mut self, event: KeyEvent) {
        if event.modifiers.is_empty() {
            match event.code {
                KeyCode::Char(c) =>
                    if c.is_whitespace() {
                        self.cursor.escape_to_normal();
                    } else {
                        self.cursor.input(c);
                    },
                _ =>
                    (),
            }
        }
    }

    pub fn handle_key_event_strlit(&mut self, event: KeyEvent) {
        if event.modifiers.is_empty() {
            match event.code {
                KeyCode::Char(c) =>
                    if c == '"' {
                        self.cursor.escape_to_normal();
                    } else {
                        self.cursor.input(c);
                    },
                _ =>
                    (),
            }
        }
    }

    pub fn render<W: Write>(&self, term: &mut Terminal<W>) {
        let size = match term.get(Value::TerminalSize) {
            Ok(Retrieved::TerminalSize(width, height)) =>
                Size { width: width as usize, height: height as usize },
            _ =>
                panic!(),
        };
        self.layout().display(Pos { x: 0, y: 0 }, size, term);
    }
}

impl Pretty for Shell {
    fn layout(&self) -> Layout {
        let cmdline = Layout::Weight(Box::new(self.cursor.layout()), 100f64);
        let sep = Layout::HLine(Symbol {
            glyph: '~',
            foreground: Color::Grey,
            background: Color::Black,
        });
        let debugger = if self.cursor.mode() == Mode::Normal {
            let mut vm = VM::new();
            let mut trace = HashMap::new();
            vm.eval_cursor(&mut trace, Cursor::initial(self.cursor.program()));
            if let Some(snapshots) = trace.get(&self.cursor.shape()) {
                Layout::VConcat(snapshots.iter().take(16).map(|snapshot| snapshot.layout()).collect())
            } else {
                Layout::Empty
            }
        } else {
            Layout::Empty
        };
        Layout::VConcat(vec![cmdline, sep, debugger])
    }
}
