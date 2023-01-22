use std::mem;
use terminal::Color;
use crate::{
    syntax::{Expr, Program},
    pretty::{PrettyText, TextBuilder},
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Cursor {
    Edge(Program, Program),
    Quote(Program, Box<Cursor>, Program),
    Ident(Program, usize, Vec<char>, Program),
    StrLit(Program, usize, Vec<char>, Program),
    NumLit(Program, Option<i64>, Program),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CursorShape {
    Edge(usize, usize),
    Quote(usize, Box<CursorShape>, usize),
    Ident(usize, usize),
    StrLit(usize, usize),
    NumLit(usize, usize),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Ident,
    StrLit,
    NumLit,
}

impl Default for Cursor {
    fn default() -> Self {
        Self::empty()
    }
}

impl Cursor {
    pub fn empty() -> Self {
        Self::Edge(Vec::new(), Vec::new())
    }

    pub fn empty_ident() -> Self {
        Self::Ident(Vec::new(), 0, Vec::new(), Vec::new())
    }

    pub fn empty_str_lit() -> Self {
        Self::StrLit(Vec::new(), 0, Vec::new(), Vec::new())
    }

    pub fn empty_num_lit() -> Self {
        Self::NumLit(Vec::new(), None, Vec::new())
    }

    pub fn empty_quote() -> Self {
        Self::Quote(Vec::new(), Box::new(Self::empty()), Vec::new())
    }

    pub fn initial(program: Program) -> Self {
        Self::Edge(Vec::new(), program)
    }

    pub fn shape(&self) -> CursorShape {
        match self {
            Self::Edge(head, tail) => CursorShape::Edge(head.len(), tail.len()),
            Self::Quote(head, cursor, tail) => CursorShape::Quote(head.len(), Box::new(cursor.shape()), tail.len()),
            Self::Ident(head, _, _, tail) => CursorShape::Ident(head.len(), tail.len()),
            Self::StrLit(head, _, _, tail) => CursorShape::StrLit(head.len(), tail.len()),
            Self::NumLit(head, _, tail) => CursorShape::NumLit(head.len(), tail.len()),
        }
    }

    pub fn mode(&self) -> Mode {
        match self {
            Self::Edge(_, _) => Mode::Normal,
            Self::Quote(_, cursor, _) => cursor.mode(),
            Self::Ident(_, _, _, _) => Mode::Ident,
            Self::StrLit(_, _, _, _) => Mode::StrLit,
            Self::NumLit(_, _, _) => Mode::NumLit,
        }
    }

    pub fn program(&self) -> Program {
        match self {
            Self::Edge(head, tail) => {
                let mut program = head.clone();
                program.append(&mut tail.clone());
                program
            },
            Self::Quote(head, cursor, tail) => {
                let mut program = head.clone();
                program.push(Expr::Quote(cursor.program()));
                program.append(&mut tail.clone());
                program
            },
            _ => {
                panic!();
            },
        }
    }

    pub fn local_program(&self) -> Program {
        match self {
            Self::Edge(head, tail) => {
                let mut program = head.clone();
                program.append(&mut tail.clone());
                program
            },
            Self::Quote(_, cursor, _) => {
                cursor.local_program()
            },
            _ => {
                panic!();
            },
        }
    }

    pub fn next_expr(&self) -> Option<&Expr> {
        match self {
            Self::Edge(_, tail) => tail.get(0),
            Self::Quote(_, cursor, _) => cursor.next_expr(),
            Self::Ident(_, _, _, tail) => tail.get(0),
            Self::StrLit(_, _, _, tail) => tail.get(0),
            Self::NumLit(_, _, tail) => tail.get(0),
        }
    }

    pub fn escape_to_normal(&mut self) {
        *self = match mem::take(self) {
            Self::Edge(head, tail) => {
                Self::Edge(head, tail)
            },
            Self::Quote(head, mut cursor, tail) => {
                cursor.escape_to_normal();
                Self::Quote(head, cursor, tail)
            },
            Self::Ident(mut head, _, s, tail) => {
                head.push(Expr::Ident(s.into_iter().collect()));
                Self::Edge(head, tail)
            },
            Self::StrLit(mut head, _, s, tail) => {
                head.push(Expr::StrLit(s.into_iter().collect()));
                Self::Edge(head, tail)
            },
            Self::NumLit(mut head, n, tail) => {
                match n {
                    Some(n) => head.push(Expr::NumLit(n)),
                    None => (),
                }
                Self::Edge(head, tail)
            },
        };
    }

    pub fn move_left(&mut self) {
        match self {
            Self::Edge(head, tail) => {
                if let Some(expr) = head.pop() {
                    tail.insert(0, expr);
                }
            },
            Self::Quote(_, cursor, _) => {
                cursor.move_left();
            },
            Self::Ident(_, n, _, _) => {
                if *n > 0 {
                    *n -= 1;
                }
            },
            Self::StrLit(_, n, _, _) => {
                if *n > 0 {
                    *n -= 1;
                }
            },
            Self::NumLit(_, _, _) => {},
        }
    }

    pub fn move_very_left(&mut self) {
        match self {
            Self::Edge(ref mut head, ref mut tail) => {
                head.append(tail);
                mem::swap(head, tail);
            },
            Self::Quote(_, cursor, _) => {
                cursor.move_very_left();
            },
            Self::Ident(_, n, _, _) => {
                *n = 0;
            },
            Self::StrLit(_, n, _, _) => {
                *n = 0;
            },
            Self::NumLit(_, _, _) => {},
        }
    }

    pub fn move_right(&mut self) {
        match self {
            Self::Edge(head, tail) => {
                if !tail.is_empty() {
                    let expr = tail.remove(0);
                    head.push(expr);
                }
            },
            Self::Quote(_, cursor, _) => {
                cursor.move_right();
            },
            Self::Ident(_, n, s, _) => {
                if *n < s.len() {
                    *n += 1;
                }
            },
            Self::StrLit(_, n, s, _) => {
                if *n < s.len() {
                    *n += 1;
                }
            },
            Self::NumLit(_, _, _) => {},
        }
    }

    pub fn move_up(&mut self) {
        *self = match mem::take(self) {
            Self::Edge(mut head, tail) => {
                match head.pop() {
                    Some(Expr::Quote(program)) => {
                        Self::Quote(head, Box::new(Self::Edge(program, Vec::new())), tail)
                    },
                    Some(expr) => {
                        head.push(expr);
                        Self::Edge(head, tail)
                    },
                    None => {
                        Self::Edge(head, tail)
                    },
                }
            },
            Self::Quote(head, mut cursor, tail) => {
                cursor.move_up();
                Self::Quote(head, cursor, tail)
            },
            cursor => {
                cursor
            },
        }
    }

    pub fn move_out(&mut self) {
        *self = match mem::take(self) {
            Self::Quote(mut head, mut cursor, tail) => {
                if let Self::Quote(_, _, _) = *cursor {
                    cursor.move_out();
                    Self::Quote(head, cursor, tail)
                } else {
                    head.push(Expr::Quote(cursor.program()));
                    Self::Edge(head, tail)
                }
            },
            cursor => {
                cursor
            },
        }
    }

    pub fn insert(&mut self, subst: Cursor) {
        *self = match (mem::take(self), subst) {
            (Self::Edge(mut head, mut tail), Self::Edge(mut shead, mut stail)) => {
                head.append(&mut shead);
                stail.append(&mut tail);
                Self::Edge(head, stail)
            },
            (Self::Edge(mut head, mut tail), Self::Quote(mut shead, cursor, mut stail)) => {
                head.append(&mut shead);
                stail.append(&mut tail);
                Self::Quote(head, cursor, stail)
            },
            (Self::Edge(mut head, mut tail), Self::Ident(mut shead, n, s, mut stail)) => {
                head.append(&mut shead);
                stail.append(&mut tail);
                Self::Ident(head, n, s, stail)
            },
            (Self::Edge(mut head, mut tail), Self::StrLit(mut shead, n, s, mut stail)) => {
                head.append(&mut shead);
                stail.append(&mut tail);
                Self::StrLit(head, n, s, stail)
            },
            (Self::Edge(mut head, mut tail), Self::NumLit(mut shead, n, mut stail)) => {
                head.append(&mut shead);
                stail.append(&mut tail);
                Self::NumLit(head, n, stail)
            },
            (Self::Quote(head, mut cursor, tail), subst) => {
                cursor.insert(subst);
                Self::Quote(head, cursor, tail)
            },
            (_, _) => {
                panic!()
            }
        };
    }

    pub fn delete_before(&mut self) {
        match self {
            Self::Edge(head, _) => {
                let _ = head.pop();
            },
            Self::Quote(_, cursor, _) => {
                cursor.delete_before();
            },
            _ => {
                panic!();
            },
        }
    }

    pub fn input(&mut self, c: char) {
        match self {
            Self::Edge(_, _) =>
                panic!(),
            Self::Quote(_, cursor, _) =>
                cursor.input(c),
            Self::Ident(_, n, s, _) => {
                s.insert(*n, c);
                *n += 1;
            },
            Self::StrLit(_, n, s, _) => {
                s.insert(*n, c);
                *n += 1;
            },
            Self::NumLit(_, n, _) => {
                if let Some(digit) = c.to_digit(10) {
                    *n = Some(10 * n.unwrap_or(0) + digit as i64);
                }
            },
        }
    }
}

impl PrettyText for Cursor {
    fn get_text(&self, text: &mut TextBuilder) {
        match self {
            Self::Edge(head, tail) => {
                head.get_text(text);
                text.write_str(Color::Blue, Color::Blue, " ");
                tail.get_text(text);
            },
            Self::Quote(head, cursor, tail) => {
                head.get_text(text);
                text.write_str_default(" {");
                cursor.get_text(text);
                text.write_str_default("} ");
                tail.get_text(text);
            },
            Self::Ident(head, n, s, tail) => {
                head.get_text(text);
                if !head.is_empty() {
                    text.write_str_default(" ");
                }
                text.write_str(Color::Red, Color::Black, &s[.. *n].iter().collect::<String>());
                text.write_str(Color::Magenta, Color::Magenta, " ");
                text.write_str(Color::Red, Color::Black, &s[*n ..].iter().collect::<String>());
                if !tail.is_empty() {
                    text.write_str_default(" ");
                }
                tail.get_text(text);
            },
            Self::StrLit(head, n, s, tail) => {
                head.get_text(text);
                if !head.is_empty() {
                    text.write_str_default(" ");
                }
                text.write_str(Color::Green, Color::Black, &s[.. *n].iter().collect::<String>());
                text.write_str(Color::Magenta, Color::Magenta, " ");
                text.write_str(Color::Green, Color::Black, &s[*n ..].iter().collect::<String>());
                if !tail.is_empty() {
                    text.write_str_default(" ");
                }
                tail.get_text(text);
            },
            Self::NumLit(head, n, tail) => {
                head.get_text(text);
                if !head.is_empty() {
                    text.write_str_default(" ");
                }
                match n {
                    Some(n) => {
                        text.write_str(Color::Green, Color::Black, &format!("{n}"));
                    },
                    None => {
                        text.write_str(Color::Green, Color::Black, "0");
                    },
                }
                text.write_str(Color::Magenta, Color::Magenta, " ");
                if !tail.is_empty() {
                    text.write_str_default(" ");
                }
                tail.get_text(text);
            },
        }
    }
}
