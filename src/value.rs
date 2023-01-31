use std::sync::Arc;
use std::iter::{zip, once};
use num_bigint::BigInt;
use num_traits::cast::ToPrimitive;
use terminal::Color;
use crate::{
    polyset::Polyset,
    editor::Cursor,
    pretty::{PrettyText, TextBuilder},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Shape {
    Void,
    Any,
    Char,
    Num,
    Tuple(Vec<Shape>),
    Array(Box<Shape>, usize),
    List(Box<Shape>),
    Set(Box<Shape>),
    Quote,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Val {
    Num(BigInt),
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

impl Shape {
    pub fn union(self, that: Shape) -> Shape {
        match (self, that) {
            (Self::Void, shape2) => shape2,
            (shape1, Self::Void) => shape1,
            (Self::Any, _) => Self::Any,
            (_, Self::Any) => Self::Any,
            (Self::Char, Self::Char) => Self::Char,
            (Self::Num, Self::Num) => Self::Num,
            (Self::Tuple(shapes1), Self::Tuple(shapes2)) => {
                if shapes1.len() == shapes2.len() {
                    Self::Tuple(zip(shapes1.into_iter(), shapes2.into_iter()).map(|(s1, s2)| s1.union(s2)).collect())
                } else {
                    let shape1 = shapes1.into_iter().fold(Self::Void, Self::union);
                    let shape2 = shapes2.into_iter().fold(Self::Void, Self::union);
                    shape1.union(shape2)
                }
            },
            (Self::Tuple(shapes1), Self::Array(shape2, dim)) => {
                if shapes1.len() == dim {
                    Self::Array(Box::new(shapes1.into_iter().fold(*shape2, Self::union)), dim)
                } else {
                    Self::List(Box::new(shapes1.into_iter().fold(*shape2, Self::union)))
                }
            },
            (Self::Tuple(shapes1), Self::List(shape2)) => {
                Self::List(Box::new(shapes1.into_iter().fold(*shape2, Self::union)))
            },
            (Self::Array(shape1, dim), Self::Tuple(shapes2)) => {
                if shapes2.len() == dim {
                    Self::Array(Box::new(shapes2.into_iter().fold(*shape1, Self::union)), dim)
                } else {
                    Self::List(Box::new(shapes2.into_iter().fold(*shape1, Self::union)))
                }
            },
            (Self::Array(shape1, dim1), Self::Array(shape2, dim2)) => {
                if dim1 == dim2 {
                    Self::Array(Box::new(shape1.union(*shape2)), dim1)
                } else {
                    Self::List(Box::new(shape1.union(*shape2)))
                }
            },
            (Self::Array(shape1, _), Self::List(shape2)) => {
                Self::List(Box::new(shape1.union(*shape2)))
            },
            (Self::List(shape1), Self::Tuple(shapes2)) => {
                Self::List(Box::new(shapes2.into_iter().fold(*shape1, Self::union)))
            },
            (Self::List(shape1), Self::Array(shape2, _)) => {
                Self::List(Box::new(shape1.union(*shape2)))
            },
            (Self::List(shape1), Self::List(shape2)) => {
                Self::List(Box::new(shape1.union(*shape2)))
            },
            (Self::Set(shape1), Self::Set(shape2)) => {
                Self::Set(Box::new(shape1.union(*shape2)))
            },
            _ => Self::Any,
        }
    }

    pub fn is_string(&self) -> bool {
        match self {
            Self::Array(shape, _) => {
                **shape == Self::Char
            },
            Self::List(shape) => {
                **shape == Self::Char
            },
            _ => false,
        }
    }

    pub fn repr(&self) -> Value {
        match self {
            shape if shape.is_string() => Value::new_str("string"),
            Self::Void => Value::new_str("void"),
            Self::Any => Value::new_str("any"),
            Self::Char => Value::new_str("char"),
            Self::Num => Value::new_str("num"),
            Self::Tuple(shapes) => {
                let reprs: Vec<_> = once(Value::new_str("tuple")).chain(shapes.iter().map(Self::repr)).collect();
                Value::new_list(reprs)
            },
            Self::Array(shape, dim) => {
                Value::new_list(vec![
                    Value::new_str("array"),
                    shape.repr(),
                    Value::new_i64(*dim as i64),
                ])
            },
            Self::List(shape) => {
                Value::new_list(vec![
                    Value::new_str("list"),
                    shape.repr(),
                ])
            },
            Self::Set(shape) => {
                Value::new_list(vec![
                    Value::new_str("set"),
                    shape.repr(),
                ])
            },
            Self::Quote => {
                Value::new_str("quote")
            },
        }
    }
}

impl Val {
    pub fn as_i64(&self) -> Option<i64> {
        self.as_num()?.to_i64()
    }

    pub fn as_usize(&self) -> Option<usize> {
        self.as_num()?.to_usize()
    }

    pub fn as_num(&self) -> Option<BigInt> {
        match self {
            Self::Num(num) => Some(num.clone()),
            _ => None,
        }
    }

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

    pub fn as_string(&self) -> Option<String> {
        self.as_slice()?.iter().map(|v| v.as_char()).collect()
    }

    pub fn shape(&self) -> Shape {
        match self {
            Self::Num(_) => Shape::Num,
            Self::List(list) => {
                let shape = list.iter().map(Value::shape).fold(Shape::Void, Shape::union);
                if list.len() <= 8 && list.iter().any(|s| s.shape() != shape) {
                    Shape::Tuple(list.iter().map(Value::shape).collect())
                } else {
                    Shape::Array(Box::new(shape), list.len())
                }
            },
            Self::Set(set) => Shape::Set(Box::new(set.iter().map(|(v, _)| v.shape()).fold(Shape::Void, Shape::union))),
            Self::Quote(_) => Shape::Quote,
        }
    }

    fn shaped_text(&self, shape: &Shape, text: &mut TextBuilder) {
        match self {
            Val::Num(num) => {
                text.write_str(Color::Green, Color::Black, &format!("{num}"));
            },
            Val::List(values) => {
                match shape {
                    Shape::Array(elem_shape, _) | Shape::List(elem_shape) => {
                        if **elem_shape == Shape::Char {
                            let s = self.as_string().unwrap();
                            let s = if s.is_empty() {
                                "ε".to_string()
                            } else {
                                s.replace('\n', "↵")
                            };
                            text.write_str(Color::Green, Color::Black, &s);
                        } else {
                            text.write_str_default("[");
                            for (i, value) in values.iter().enumerate() {
                                if i > 0 {
                                    text.write_str_default(" ");
                                }
                                value.shaped_text(elem_shape, text);
                            }
                            text.write_str_default("]");
                        }
                    },
                    Shape::Tuple(shapes) => {
                        text.write_str_default("[");
                        for (i, (value, elem_shape)) in values.iter().zip(shapes.iter()).enumerate() {
                            if i > 0 {
                                text.write_str_default(" ");
                            }
                            value.shaped_text(elem_shape, text);
                        }
                        text.write_str_default("]");
                    },
                    _ => {
                        self.get_text(text);
                    },
                }
            },
            Val::Set(values) => {
                match shape {
                    Shape::Set(item_shape) => {
                        text.write_str_default("⟨");
                        for (i, (value, n)) in values.iter().enumerate() {
                            if i > 0 {
                                text.write_str_default(" ");
                            }
                            value.shaped_text(item_shape, text);
                            if *n != 1 {
                                text.write_str_default(&format!(":{n}"));
                            }
                        }
                        text.write_str_default("⟩");
                    },
                    _ => {
                        self.get_text(text);
                    },
                }
            },
            Val::Quote(cursor) => {
                text.write_str_default("{");
                cursor.local_program().get_text(text);
                text.write_str_default("}");
            },
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

    pub fn new_i64(val: i64) -> Self {
        Self::Num(val)
    }

    pub fn new_num(val: BigInt) -> Self {
        match val.to_i64() {
            Some(n) => Self::Num(n),
            None => Self::new_val(Val::Num(val)),
        }
    }

    pub fn new_str(val: &str) -> Self {
        Self::new_list(val.chars().map(Value::new_char).collect())
    }

    pub fn new_bool(val: bool) -> Self {
        Self::new_i64(if val { 1 } else { 0 })
    }

    pub fn new_list(val: Vec<Value>) -> Self {
        Self::new_val(Val::List(val))
    }

    pub fn new_set(val: Polyset<Value>) -> Self {
        Self::new_val(Val::Set(val))
    }

    pub fn new_quote(val: Cursor) -> Self {
        Self::new_val(Val::Quote(val))
    }

    pub fn new_val(val: Val) -> Self {
        Self::Ptr(Arc::new(val))
    }

    pub fn as_char(&self) -> Option<char> {
        match self {
            Self::Char(c) => Some(*c),
            _ => None,
        }
    }

    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Self::Num(n) => Some(*n),
            _ => self.as_ptr()?.as_i64(),
        }
    }

    pub fn as_usize(&self) -> Option<usize> {
        match self {
            Self::Num(n) => (*n).try_into().ok(),
            _ => self.as_ptr()?.as_usize(),
        }
    }

    pub fn as_num(&self) -> Option<BigInt> {
        match self {
            Self::Num(n) => Some((*n).into()),
            _ => self.as_ptr()?.as_num(),
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self.as_i64()? {
            0 => Some(false),
            1 => Some(true),
            _ => None,
        }
    }

    fn as_ptr(&self) -> Option<&Val> {
        match self {
            Self::Ptr(val) => Some(val),
            _ => None,
        }
    }

    pub fn as_list(&self) -> Option<Vec<Value>> {
        self.as_ptr()?.as_list()
    }

    pub fn as_set(&self) -> Option<Polyset<Value>> {
        self.as_ptr()?.as_set()
    }

    pub fn as_quote(&self) -> Option<&Cursor> {
        self.as_ptr()?.as_quote()
    }

    pub fn as_slice(&self) -> Option<&[Value]> {
        self.as_ptr()?.as_slice()
    }

    pub fn as_string(&self) -> Option<String> {
        self.as_ptr()?.as_string()
    }

    pub fn shape(&self) -> Shape {
        match self {
            Self::Poison => Shape::Any,
            Self::Char(_) => Shape::Char,
            Self::Num(_) => Shape::Num,
            Self::Ptr(v) => v.shape(),
        }
    }

    fn shaped_text(&self, shape: &Shape, text: &mut TextBuilder) {
        match self {
            Self::Poison => {
                text.write_str(Color::Black, Color::White, "☠");
            },
            Self::Char(c) => {
                let c = match *c {
                    '\n' => '↵',
                    ' ' => '⋅',
                    c => c,
                };
                text.write_char(Color::Green, Color::Black, c);
            },
            Self::Num(n) => {
                text.write_str(Color::Green, Color::Black, &format!("{n}"));
            },
            Self::Ptr(val) => {
                val.shaped_text(shape, text);
            },
        }
    }
}

impl PrettyText for Val {
    fn get_text(&self, text: &mut TextBuilder) {
        self.shaped_text(&self.shape(), text);
    }
}

impl PrettyText for Value {
    fn get_text(&self, text: &mut TextBuilder) {
        self.shaped_text(&self.shape(), text);
    }
}
