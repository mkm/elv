use std::sync::Arc;
use std::borrow::Borrow;
use terminal::Color;
use crate::{
    polyset::Polyset,
    editor::Cursor,
    pretty::{PrettyText, TextBuilder},
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

impl PrettyText for Val {
    fn get_text(&self, text: &mut TextBuilder) {
        match self {
            Val::Poison => {
                text.write_str(Color::Black, Color::White, "☠");
            },
            Val::Str(s) => {
                let output = s.replace('\n', "⋅");
                if output.is_empty() {
                    text.write_str(Color::Green, Color::Black, "ε");
                } else {
                    text.write_str(Color::Green, Color::Black, &output);
                }
            },
            Val::Num(n) => {
                text.write_str(Color::Green, Color::Black, &format!("{n}"));
            },
            Val::List(values) => {
                text.write_str_default("[");
                for (i, value) in values.iter().enumerate() {
                    if i > 0 {
                        text.write_str_default(" ");
                    }
                    value.get_text(text);
                }
                text.write_str_default("]");
            },
            Val::Set(values) => {
                text.write_str_default("[");
                for (i, (value, n)) in values.iter().enumerate() {
                    if i > 0 {
                        text.write_str_default(" ");
                    }
                    value.get_text(text);
                    if *n != 1 {
                        text.write_str_default(&format!(":{n}"));
                    }
                }
                text.write_str_default("]");
            },
            Val::Quote(cursor) => {
                text.write_str_default("{");
                cursor.local_program().get_text(text);
                text.write_str_default("}");
            },
        }
    }
}

impl PrettyText for Value {
    fn get_text(&self, text: &mut TextBuilder) {
        self.0.get_text(text);
    }
}
