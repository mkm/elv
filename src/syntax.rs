use std::io::Write;
use terminal::{Terminal, Action, Color};
use crate::pretty::{Pretty, Size, Rect, Req};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Expr {
    Ident(String),
    StrLit(String),
    Quote(Program),
}

pub type Program = Vec<Expr>;

impl Expr {
    fn pretty_size(&self) -> usize {
        match self {
            Expr::Ident(s) => s.chars().count(),
            Expr::StrLit(s) => s.chars().count(),
            Expr::Quote(program) => pretty_program_size(program),
        }
    }
}

fn pretty_program_size(program: &Program) -> usize {
    if program.is_empty() {
        0
    } else {
        program.iter().map(|expr| expr.pretty_size()).sum::<usize>() + program.len() - 1
    }
}

impl Pretty for Expr {
    fn requirements(&self) -> Req {
        let space = self.pretty_size();
        Req {
            min_space: 1,
            wanted_space: space,
            min_size: Size { width: 1, height: 1 },
            wanted_size: Size { width: space, height: 1 },
        }
    }

    fn pretty<W: Write>(&self, region: Rect, term: &mut Terminal<W>) {
        match self {
            Expr::Ident(s) => {
                term.batch(Action::SetForegroundColor(Color::Red)).unwrap();
                if s.is_empty() {
                    write!(term, "␣").unwrap();
                } else {
                    write!(term, "{}", s).unwrap();
                }
                term.batch(Action::ResetColor).unwrap();
            },
            Expr::StrLit(s) => {
                term.batch(Action::SetForegroundColor(Color::Green)).unwrap();
                if s.is_empty() {
                    write!(term, "ε").unwrap();
                } else {
                    write!(term, "{}", s).unwrap();
                }
                term.batch(Action::ResetColor).unwrap();
            },
            Expr::Quote(program) => {
                write!(term, "{{").unwrap();
                program.pretty(region, term);
                write!(term, "}}").unwrap();
            },
        }
    }
}

impl Pretty for Program {
    fn requirements(&self) -> Req {
        let space = pretty_program_size(self);
        Req {
            min_space: 1,
            wanted_space: space,
            min_size: Size { width: 1, height: 1 },
            wanted_size: Size { width: space, height: 1 },
        }
    }

    fn pretty<W: Write>(&self, region: Rect, term: &mut Terminal<W>) {
        let mut space = false;
        for expr in self {
            if space {
                write!(term, " ").unwrap();
            }
            expr.pretty(region, term);
            space = true;
        }
    }
}
