use std::io::Write;
use std::sync::Arc;
use std::borrow::Borrow;
use terminal::{Terminal, Action, Color};
use crate::{
    polyset::Polyset,
    editor::Cursor,
    pretty::{Pretty, Size, Rect, Req},
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Val {
    Poison,
    Str(String),
    Num(i64),
    List(Vec<Value>),
    Set(Polyset<Value>),
    Quote(Cursor),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Value(Arc<Val>);

impl Value {
    pub fn new_poison() -> Self {
        Self(Arc::new(Val::Poison))
    }

    pub fn new_str(val: String) -> Self {
        Self(Arc::new(Val::Str(val)))
    }

    pub fn new_num(val: i64) -> Self {
        Self(Arc::new(Val::Num(val)))
    }

    pub fn new_bool(val: bool) -> Self {
        Self::new_num(if val { 1 } else { 0 })
    }

    pub fn new_list(val: Vec<Value>) -> Self {
        Self(Arc::new(Val::List(val)))
    }

    pub fn new_set(val: Polyset<Value>) -> Self {
        Self(Arc::new(Val::Set(val)))
    }

    pub fn new_quote(val: Cursor) -> Self {
        Self(Arc::new(Val::Quote(val)))
    }

    pub fn as_str(&self) -> Option<&str> {
        match self.0.borrow() {
            Val::Str(s) => Some(s.as_str()),
            _ => None,
        }
    }

    pub fn as_char(&self) -> Option<char> {
        let s = self.as_str()?;
        if s.len() == 1 {
            s.chars().next()
        } else {
            None
        }
    }

    pub fn as_num(&self) -> Option<i64> {
        match self.0.borrow() {
            Val::Str(s) => s.parse().ok(),
            Val::Num(n) => Some(*n),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self.as_num()? {
            0 => Some(false),
            1 => Some(true),
            _ => None,
        }
    }

    pub fn as_list(&self) -> Option<Vec<Value>> {
        match self.0.borrow() {
            Val::Str(s) => Some(s.chars().map(|c| Value::new_str(String::from(c))).collect::<Vec<_>>()),
            Val::List(v) => Some(v.clone()),
            _ => None,
        }
    }

    pub fn as_set(&self) -> Option<Polyset<Value>> {
        match self.0.borrow() {
            Val::Set(s) => Some(s.clone()),
            _ => Some(Polyset::from_vec(self.as_list()?)),
        }
    }

    pub fn as_quote(&self) -> Option<&Cursor> {
        match self.0.borrow() {
            Val::Quote(cursor) => Some(&cursor),
            _ => None,
        }
    }
}

impl Pretty for Val {
    fn requirements(&self) -> Req {
        Req {
            min_space: 1,
            wanted_space: 1,
            min_size: Size { width: 1, height: 1 },
            wanted_size: Size { width: 1, height: 1 },
        }
    }

    fn pretty<W: Write>(&self, region: Rect, term: &mut Terminal<W>) {
        match self {
            Val::Poison => {
                write!(term, "☠").unwrap();
            },
            Val::Str(s) => {
                term.batch(Action::SetForegroundColor(Color::Green)).unwrap();
                let mut buf: Vec<_> = s.replace('\n', "⋅").chars().collect();
                buf.truncate(350);
                if buf.is_empty() {
                    buf.push('ε');
                }
                if buf.len() == 350 {
                    buf.push('…');
                }
                let output: String = buf.into_iter().collect();
                write!(term, "{output}").unwrap();
                term.batch(Action::ResetColor).unwrap();
            },
            Val::Num(n) => {
                term.batch(Action::SetForegroundColor(Color::Green)).unwrap();
                write!(term, "{n}").unwrap();
                term.batch(Action::ResetColor).unwrap();
            },
            Val::List(values) => {
                write!(term, "[").unwrap();
                for (i, value) in values.iter().enumerate() {
                    if i > 0 {
                        write!(term, " ").unwrap();
                    }
                    if i > 20 {
                        write!(term, "…").unwrap();
                        break;
                    }
                    value.pretty(region, term);
                }
                write!(term, "]").unwrap();
            },
            Val::Set(values) => {
                write!(term, "[").unwrap();
                for (i, (value, n)) in values.iter().enumerate() {
                    if i > 0 {
                        write!(term, " ").unwrap();
                    }
                    if i > 20 {
                        write!(term, "…").unwrap();
                        break;
                    }
                    value.pretty(region, term);
                    if *n != 1 {
                        write!(term, ":{n}").unwrap();
                    }
                }
                write!(term, "]").unwrap();
            },
            Val::Quote(cursor) => {
                write!(term, "{{").unwrap();
                cursor.local_program().pretty(region, term);
                write!(term, "}}").unwrap();
            },
        }
    }
}

impl Pretty for Value {
    fn requirements(&self) -> Req {
        Req {
            min_space: 1,
            wanted_space: 1,
            min_size: Size { width: 1, height: 1 },
            wanted_size: Size { width: 1, height: 1 },
        }
    }

    fn pretty<W: Write>(&self, region: Rect, term: &mut Terminal<W>) {
        self.0.pretty(region, term);
    }
}
