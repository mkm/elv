use num_bigint::BigInt;
use terminal::Color;
use crate::pretty::{PrettyText, TextBuilder};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Expr {
    Ident(String),
    StrLit(String),
    NumLit(BigInt),
    Quote(Program),
}

pub type Program = Vec<Expr>;

impl PrettyText for Expr {
    fn get_text(&self, text: &mut TextBuilder) {
        match self {
            Expr::Ident(s) => {
                if s.is_empty() {
                    text.write_str(Color::Red, Color::Black, "␣");
                } else {
                    text.write_str(Color::Red, Color::Black, &s);
                }
            },
            Expr::StrLit(s) => {
                if s.is_empty() {
                    text.write_str(Color::Green, Color::Black, "ε");
                } else {
                    text.write_str(Color::Green, Color::Black, &s);
                }
            },
            Expr::NumLit(n) => {
                text.write_str(Color::Green, Color::Black, &format!("{n}"));
            },
            Expr::Quote(program) => {
                text.write_str_default("{");
                program.get_text(text);
                text.write_str_default("}");
            },
        }
    }
}

impl PrettyText for Program {
    fn get_text(&self, text: &mut TextBuilder) {
        for (i, expr) in self.iter().enumerate() {
            if i > 0 {
                text.write_str_default(" ");
            }
            expr.get_text(text);
        }
    }
}
