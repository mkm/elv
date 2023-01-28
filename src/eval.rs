use std::cmp::max;
use std::collections::HashMap;
use terminal::Color;
use crate::{
    polyset::Polyset,
    syntax::{Expr},
    editor::{Cursor, CursorShape},
    value::Value,
    pretty::{Pretty, Layout},
};

#[derive(Debug, Clone)]
pub struct VM {
    parent: Option<Box<VM>>,
    stack: Vec<Value>,
}

pub type Trace = HashMap<CursorShape, Vec<VM>>;

impl VM {
    pub fn new() -> Self {
        Self {
            parent: None,
            stack: Vec::new(),
        }
    }

    pub fn new_child(&self) -> Self {
        Self {
            parent: Some(Box::new(self.clone())),
            stack: Vec::new(),
        }
    }

    fn add_snapshot(&mut self, trace: &mut Trace, key: CursorShape) {
        trace.entry(key).or_insert(Vec::new()).push(self.clone());
    }

    fn pop(&mut self) -> Option<Value> {
        self.stack.pop()
    }

    fn push(&mut self, value: Value) {
        self.stack.push(value)
    }

    fn eval_prim(&mut self, trace: &mut Trace, prim: &str) {
        let result = try {
            match prim {
                "del" => {
                    self.pop()?;
                },
                "dup" => {
                    let value = self.pop()?;
                    self.push(value.clone());
                    self.push(value);
                },
                "flip" => {
                    let fst = self.pop()?;
                    let snd = self.pop()?;
                    self.push(fst);
                    self.push(snd);
                },
                "copy" => {
                    let index = self.pop()?.as_num()? as usize;
                    let value = self.stack.get(self.stack.len() - 1 - index)?.clone();
                    self.push(value);
                },
                "move" => {
                    let offset = self.pop()?.as_num()? as usize;
                    let index = self.stack.len() - 1 - offset;
                    if index < self.stack.len() {
                        let value = self.stack.remove(index);
                        self.push(value);
                    } else {
                        None?
                    }
                },
                "sb" => {
                    let new = self.pop()?;
                    let test = self.pop()?;
                    let value = self.pop()?;
                    if value == test {
                        self.push(new);
                    } else {
                        self.push(value);
                    }
                },
                "s" => {
                    let arg3 = self.pop()?;
                    let arg2 = self.pop()?;
                    let haystack = self.pop()?.as_string()?;
                    self.push(Value::new_str(haystack.replace(&arg2.as_string()?, &arg3.as_string()?)));
                },
                "inc" => {
                    let a = self.pop()?.as_num()?;
                    self.push(Value::new_num(a + 1));
                },
                "+" => {
                    let b = self.pop()?.as_num()?;
                    let a = self.pop()?.as_num()?;
                    self.push(Value::new_num(a + b));
                },
                "*" => {
                    let b = self.pop()?.as_num()?;
                    let a = self.pop()?.as_num()?;
                    self.push(Value::new_num(a * b));
                },
                "/" => {
                    let b = self.pop()?.as_num()?;
                    let a = self.pop()?.as_num()?;
                    self.push(Value::new_num(a / b));
                },
                "==" => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.push(Value::new_bool(a == b));
                },
                "=<" => {
                    let b = self.pop()?.as_num()?;
                    let a = self.pop()?.as_num()?;
                    self.push(Value::new_bool(a <= b));
                },
                ">=" => {
                    let b = self.pop()?.as_num()?;
                    let a = self.pop()?.as_num()?;
                    self.push(Value::new_bool(a >= b));
                },
                "and" => {
                    let b = self.pop()?.as_bool()?;
                    let a = self.pop()?.as_bool()?;
                    self.push(Value::new_bool(a && b));
                },
                "or" => {
                    let b = self.pop()?.as_bool()?;
                    let a = self.pop()?.as_bool()?;
                    self.push(Value::new_bool(a || b));
                },
                "read" => {
                    let contents = std::fs::read_to_string(self.pop()?.as_string()?).ok()?;
                    self.push(Value::new_str(contents));
                },
                "lines" => {
                    let arg = self.pop()?.as_string()?;
                    let lines = arg.split('\n');
                    let mut result: Vec<_> = lines.map(|line| Value::new_str(line.to_string())).collect();
                    if result.last() == Some(&Value::new_str(String::new())) {
                        result.pop().unwrap();
                    }
                    self.push(Value::new_list(result));
                },
                "words" => {
                    let arg = self.pop()?.as_string()?;
                    let words = arg.split(|c: char| !c.is_alphanumeric());
                    self.push(Value::new_list(words.map(|word| Value::new_str(word.to_string())).collect()));
                },
                "split" => {
                    let sep = self.pop()?;
                    let list = self.pop()?.as_list()?;
                    let pieces = list.split(|v| *v == sep);
                    self.push(Value::new_list(pieces.map(|piece| Value::new_list(piece.into_iter().cloned().collect())).collect()));
                },
                "splitat" => {
                    let mut index = self.pop()?.as_num()?;
                    let arg = self.pop()?;
                    let mut list: Vec<_> = arg.as_list()?;
                    if index < 0 {
                        index = list.len() as i64 + index;
                    }
                    self.push(Value::new_list(list.split_off(index as usize)));
                    self.push(Value::new_list(list));
                },
                "take" => {
                    let mut count = self.pop()?.as_num()?;
                    let mut list: Vec<_> = self.pop()?.as_list()?;
                    if count < 0 {
                        count = max(0, list.len() as i64 + count);
                    }
                    list.truncate(count as usize);
                    self.push(Value::new_list(list))
                },
                "irange" => {
                    let upper = self.pop()?.as_num()?;
                    let lower = self.pop()?.as_num()?;
                    self.push(Value::new_list((lower ..= upper).map(|n| Value::new_num(n)).collect()));
                },
                "crange" => {
                    let upper = self.pop()?.as_char()?;
                    let lower = self.pop()?.as_char()?;
                    self.push(Value::new_list((lower ..= upper).map(|c| Value::new_str(String::from(c))).collect()));
                },
                "indexed" => {
                    let list = self.pop()?.as_list()?;
                    let indexed = list.iter().enumerate().map(|(i, v)| {
                        Value::new_list(vec![Value::new_num(i as i64), v.clone()])
                    });
                    self.push(Value::new_list(indexed.collect()));
                },
                "num" => {
                    let arg = self.pop()?.as_string()?;
                    match arg.parse() {
                        Ok(n) => self.push(Value::new_num(n)),
                        Err(_) => self.push(Value::new_poison()),
                    }
                },
                "collect" => {
                    let arg = self.pop()?;
                    let cursor = arg.as_quote()?;
                    let mut vm = self.new_child();
                    vm.eval_cursor(trace, cursor.clone());
                    self.push(Value::new_list(vm.stack))
                },
                "each" => {
                    let list = self.pop()?.as_list()?;
                    for value in list.into_iter().rev() {
                        self.push(value)
                    }
                },
                "set" => {
                    let arg = self.pop()?;
                    let list = arg.as_list()?;
                    self.push(Value::new_set(Polyset::from_vec(list)));
                },
                "nub" => {
                    let set = self.pop()?.as_set()?;
                    self.push(Value::new_list(set.keys().cloned().collect()))
                },
                "iota" => {
                    let count = self.pop()?.as_num()?;
                    self.push(Value::new_list((0 .. count).map(|i| Value::new_num(i)).collect()));
                },
                "chunks" => {
                    let size = self.pop()?.as_num()?;
                    let list = self.pop()?.as_list()?;
                    self.push(Value::new_list(list.chunks(size as usize).map(|chunk| Value::new_list(chunk.to_vec())).collect()));
                },
                "frames" => {
                    let size = self.pop()?.as_num()? as usize;
                    let list = self.pop()?.as_list()?;
                    if size > list.len() {
                        self.push(Value::new_list(Vec::new()));
                    } else {
                        let mut result = Vec::new();
                        for i in 0 .. list.len() - size {
                            result.push(Value::new_list(list[i .. i + size].to_vec()));
                        }
                        self.push(Value::new_list(result));
                    }
                },
                "len" => {
                    let arg = self.pop()?;
                    self.push(Value::new_num(arg.as_slice()?.len() as i64));
                },
                "sum" => {
                    let arg = self.pop()?;
                    let mut result: i64 = 0;
                    for value in arg.as_list()? {
                        result += value.as_num()?;
                    }
                    self.push(Value::new_num(result));
                },
                "max" => {
                    let arg = self.pop()?;
                    let mut result: i64 = i64::MIN;
                    for value in arg.as_list()? {
                        result = max(result, value.as_num()?);
                    }
                    self.push(Value::new_num(result));
                },
                "rsort" => {
                    let arg = self.pop()?;
                    let mut list: Vec<_> = arg.as_list()?;
                    list.sort_by(|a, b| b.cmp(a));
                    self.push(Value::new_list(list));
                },
                "append" => {
                    let mut b = self.pop()?.as_list()?;
                    let mut a = self.pop()?.as_list()?;
                    a.append(&mut b);
                    self.push(Value::new_list(a));
                },
                "find" => {
                    let table = self.pop()?.as_list()?;
                    let needle = self.pop()?;
                    match table.iter().position(|v| *v == needle) {
                        None => self.push(Value::new_poison()),
                        Some(i) => self.push(Value::new_num(i as i64)),
                    }
                },
                "union" => {
                    let a = self.pop()?.as_set()?;
                    let b = self.pop()?.as_set()?;
                    self.push(Value::new_set(a.union(b)));
                },
                "join" => {
                    let a = self.pop()?.as_set()?;
                    let b = self.pop()?.as_set()?;
                    self.push(Value::new_set(a.join(b)));
                },
                "map" => {
                    let arg2 = self.pop()?;
                    let arg1 = self.pop()?;
                    let list = arg1.as_list()?;
                    let cursor = arg2.as_quote()?;
                    let mut result = Vec::new();
                    for value in list {
                        let mut vm = self.new_child();
                        vm.stack.push(value.clone());
                        vm.eval_cursor(trace, cursor.clone());
                        result.append(&mut vm.stack);
                    }
                    self.push(Value::new_list(result));
                },
                "under" => {
                    let count = self.pop()?.as_num()? as usize;
                    let cursor = self.pop()?.as_quote()?.clone();
                    let index = self.stack.len() - count;
                    if index > self.stack.len() {
                        None?
                    }
                    let mut temp = self.stack.split_off(index);
                    self.eval_cursor(trace, cursor);
                    self.stack.append(&mut temp);
                },
                "shape" => {
                    let arg = self.pop()?;
                    self.push(arg.shape().repr());
                },
                _ => {
                    None?;
                },
            }
        };
        match result {
            None => self.push(Value::new_poison()),
            Some(()) => (),
        }
    }

    pub fn eval_cursor(&mut self, trace: &mut Trace, mut cursor: Cursor) {
        self.add_snapshot(trace, cursor.shape());
        while let Some(expr) = cursor.next_expr().cloned() {
            cursor.move_right();
            match expr {
                Expr::Ident(prim) => {
                    self.eval_prim(trace, prim.as_str());
                },
                Expr::StrLit(s) => {
                    self.push(Value::new_str(s.clone()));
                },
                Expr::NumLit(n) => {
                    self.push(Value::new_num(n));
                },
                Expr::Quote(_) => {
                    let mut quote_cursor = cursor.clone();
                    quote_cursor.move_up();
                    quote_cursor.move_very_left();
                    self.push(Value::new_quote(quote_cursor));
                },
            }
            self.add_snapshot(trace, cursor.shape());
        }
    }
}

impl Pretty for VM {
    fn layout(&self) -> Layout {
        let layout = Layout::VConcat(self.stack.iter().enumerate().map(|(index, item)| {
            let offset = self.stack.len() - index - 1;
            let header = Layout::ExactWidth(Box::new(Layout::mk_text(Color::Cyan, Color::Black, &format!("{offset}"))), 4);
            Layout::Diminish(Box::new(Layout::HConcat(vec![header, item.layout()])))
        }).collect());
        if let Some(parent) = &self.parent {
            Layout::VConcat(vec![parent.layout(), layout])
        } else {
            layout
        }
    }
}
