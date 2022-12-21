use std::io::Write;
use std::cmp::max;
use terminal::Terminal;
use crate::{
    polyset::Polyset,
    syntax::{Expr, Program},
    value::Value,
    pretty::{Pretty, Size, Rect, Req},
};

pub struct VM {
    stack: Vec<Value>,
}

impl VM {
    pub fn new() -> Self {
        VM {
            stack: Vec::new(),
        }
    }

    fn pop(&mut self) -> Option<Value> {
        self.stack.pop()
    }

    fn push(&mut self, value: Value) {
        self.stack.push(value)
    }

    fn eval_prim(&mut self, prim: &str) {
        let result = try {
            match prim {
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
                    let haystack = self.pop()?.as_str()?.to_string();
                    self.push(Value::new_str(haystack.replace(arg2.as_str()?, arg3.as_str()?)));
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
                "/" => {
                    let b = self.pop()?.as_num()?;
                    let a = self.pop()?.as_num()?;
                    self.push(Value::new_num(a / b));
                },
                "read" => {
                    let contents = std::fs::read_to_string(self.pop()?.as_str()?).ok()?;
                    self.push(Value::new_str(contents));
                },
                "lines" => {
                    let arg = self.pop()?;
                    let lines = arg.as_str()?.split('\n');
                    let mut result: Vec<_> = lines.map(|line| Value::new_str(line.to_string())).collect();
                    if result.last() == Some(&Value::new_str(String::new())) {
                        result.pop().unwrap();
                    }
                    self.push(Value::new_list(result));
                },
                "words" => {
                    let arg = self.pop()?;
                    let words = arg.as_str()?.split(|c: char| !c.is_alphanumeric());
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
                "list" => {
                    let arg = self.pop()?;
                    let prog = arg.as_quote()?;
                    let mut vm = Self::new();
                    vm.eval_program(prog);
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
                "len" => {
                    let arg = self.pop()?;
                    if let Some(s) = arg.as_str() {
                        self.push(Value::new_num(s.len() as i64));
                    } else {
                        self.push(Value::new_num(arg.as_list()?.len() as i64));
                    }
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
                    let prog = arg2.as_quote()?;
                    let mut result = Vec::new();
                    for value in list {
                        let mut vm = Self::new();
                        vm.stack.push(value.clone());
                        vm.eval_program(prog);
                        result.append(&mut vm.stack);
                    }
                    self.push(Value::new_list(result));
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

    pub fn eval_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Ident(prim) => {
                self.eval_prim(prim.as_str());
            },
            Expr::StrLit(s) => {
                self.push(Value::new_str(s.clone()));
            },
            Expr::Quote(program) => {
                self.push(Value::new_quote(program.clone()));
            }
        }
    }

    pub fn eval_program(&mut self, program: &Program) {
        for expr in program {
            self.eval_expr(expr);
        }
    }
}

impl Pretty for VM {
    fn requirements(&self) -> Req {
        Req {
            min_space: 1,
            wanted_space: 1,
            min_size: Size { width: 1, height: 1 },
            wanted_size: Size { width: 1, height: 1 },
        }
    }

    fn pretty<W: Write>(&self, region: Rect, term: &mut Terminal<W>) {
        for item in &self.stack {
            write!(term, "# ").unwrap();
            item.pretty(region, term);
            write!(term, "\r\n").unwrap();
        }
    }
}
