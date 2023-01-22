use std::sync::Arc;
use terminal::Color;
use crate::{
    polyset::Polyset,
    editor::Cursor,
    pretty::{PrettyText, TextBuilder},
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Val {
    List(Vec<Value>),
    Set(Polyset<Value>),
    Quote(Cursor),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Value {
    Poison,
    Char(char),
    Num(i64),
    Ptr(Arc<Val>),
}

impl Val {
    pub fn as_list(&self) -> Option<Vec<Value>> {
        match self {
            Self::List(list) => Some(list.clone()),
            _ => None,
        }
    }

    pub fn as_set(&self) -> Option<Polyset<Value>> {
        match self {
            Self::List(list) => Some(list.iter().cloned().collect()),
            Self::Set(set) => Some(set.clone()),
            _ => None,
        }
    }

    pub fn as_quote(&self) -> Option<&Cursor> {
        match self {
            Self::Quote(quote) => Some(quote),
            _ => None,
        }
    }

    pub fn as_slice(&self) -> Option<&[Value]> {
        match self {
            Self::List(list) => Some(list),
            _ => None,
        }
    }
}

impl Value {
    pub fn new_poison() -> Self {
        Self::Poison
    }

    pub fn new_char(c: char) -> Self {
        Self::Char(c)
    }

    pub fn new_num(val: i64) -> Self {
        Self::Num(val)
    }

    pub fn new_str(val: String) -> Self {
        Self::Ptr(Arc::new(Val::List(val.chars().map(Value::new_char).collect())))
    }

    pub fn new_bool(val: bool) -> Self {
        Self::new_num(if val { 1 } else { 0 })
    }

    pub fn new_list(val: Vec<Value>) -> Self {
        Self::Ptr(Arc::new(Val::List(val)))
    }

    pub fn new_set(val: Polyset<Value>) -> Self {
        Self::Ptr(Arc::new(Val::Set(val)))
    }

    pub fn new_quote(val: Cursor) -> Self {
        Self::Ptr(Arc::new(Val::Quote(val)))
    }

    pub fn as_char(&self) -> Option<char> {
        match self {
            Self::Char(c) => Some(*c),
            _ => None,
        }
    }

    pub fn as_num(&self) -> Option<i64> {
        match self {
            Self::Num(n) => Some(*n),
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
        match self {
            Self::Ptr(val) => val.as_list(),
            _ => None,
        }
    }

    pub fn as_set(&self) -> Option<Polyset<Value>> {
        match self {
            Self::Ptr(val) => val.as_set(),
            _ => None,
        }
    }

    pub fn as_quote(&self) -> Option<&Cursor> {
        match self {
            Self::Ptr(val) => val.as_quote(),
            _ => None,
        }
    }

    pub fn as_slice(&self) -> Option<&[Value]> {
        match self {
            Self::Ptr(val) => val.as_slice(),
            _ => None,
        }
    }

    pub fn as_string(&self) -> Option<String> {
        self.as_slice()?.iter().map(|v| v.as_char()).collect()
    }
}

impl PrettyText for Val {
    fn get_text(&self, text: &mut TextBuilder) {
        match self {
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
        match self {
            Self::Poison => {
                text.write_str(Color::Black, Color::White, "☠");
            },
            Self::Char(c) => {
                let s = match *c {
                    '\n' => "↵".to_string(),
                    ' ' => "⋅".to_string(),
                    c => c.to_string(),
                };
                text.write_str(Color::Green, Color::Black, &s);
            },
            Self::Num(n) => {
                text.write_str(Color::Green, Color::Black, &format!("{n}"));
            },
            Self::Ptr(val) => {
                val.get_text(text);
            },
        }
    }
}
