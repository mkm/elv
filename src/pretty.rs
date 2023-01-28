use std::collections::HashMap;
use std::io::Write;
use std::rc::Rc;
use terminal::{Terminal, Action, Color};
use unicode_width::UnicodeWidthChar;

#[derive(Debug, Clone, Copy)]
pub struct Pos {
    pub x: usize,
    pub y: usize,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct Size {
    pub width: usize,
    pub height: usize,
}

#[derive(Debug, Copy, Clone)]
pub struct Symbol {
    pub glyph: char,
    pub foreground: Color,
    pub background: Color,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum Layout {
    Empty,
    HConcat(Vec<Layout>),
    VConcat(Vec<Layout>),
    HLine(Symbol),
    VLine(Symbol),
    Text(Vec<Symbol>),
    ExactWidth(Box<Layout>, usize),
    ExactHeight(Box<Layout>, usize),
    Weight(Box<Layout>, f64),
    Diminish(Box<Layout>),
}

#[derive(Debug, Clone)]
enum EvalLayout {
    Empty,
    HConcat(Box<EvalLayout>, Box<EvalLayout>),
    VConcat(Box<EvalLayout>, Box<EvalLayout>),
    HLine(Symbol),
    VLine(Symbol),
    Text(Rc<[Symbol]>, usize),
    ExactWidth(Box<EvalLayout>, usize),
    ExactHeight(Box<EvalLayout>, usize),
    Weight(Box<EvalLayout>, f64),
    Diminish(Box<EvalLayout>),
    Cached(HashMap<Size, Option<(SizedLayout, f64)>>, Box<EvalLayout>),
}

#[derive(Debug, Clone)]
enum SizedLayout {
    Empty(Size),
    HConcat(Rc<[SizedLayout]>),
    VConcat(Rc<[SizedLayout]>),
    Fill(Symbol, Size),
    Text(Rc<[Symbol]>, Size),
}

#[derive(Debug)]
pub struct TextBuilder {
    symbols: Vec<Symbol>,
}

pub trait Pretty {
    fn layout(&self) -> Layout;
}

pub trait PrettyText {
    fn get_text(&self, text: &mut TextBuilder);
}

impl Size {
    pub fn null() -> Self {
        Self {
            width: 0,
            height: 0,
        }
    }
}

impl TextBuilder {
    pub fn new() -> Self {
        Self {
            symbols: Vec::new(),
        }
    }

    pub fn symbols(self) -> Vec<Symbol> {
        self.symbols
    }

    pub fn write_str(&mut self, foreground: Color, background: Color, s: &str) {
        for glyph in s.chars() {
            self.symbols.push(Symbol {
                glyph,
                foreground,
                background,
            });
        }
    }

    pub fn write_str_default(&mut self, s: &str) {
        self.write_str(Color::White, Color::Black, s);
    }
}

impl Layout {
    pub fn mk_text(foreground: Color, background: Color, s: &str) -> Self {
        let mut text = TextBuilder::new();
        text.write_str(foreground, background, s);
        Layout::Text(text.symbols())
    }

    fn to_eval(&self) -> EvalLayout {
        match self {
            Self::Empty => {
                EvalLayout::Empty
            },
            Self::HConcat(layouts) => {
                layouts.iter()
                    .map(|layout| layout.to_eval())
                    .reduce(|a, b| EvalLayout::Cached(HashMap::new(), Box::new(EvalLayout::HConcat(Box::new(a), Box::new(b)))))
                    .unwrap_or(EvalLayout::Empty)
            },
            Self::VConcat(layouts) => {
                layouts.iter()
                    .map(|layout| layout.to_eval())
                    .reduce(|a, b| EvalLayout::Cached(HashMap::new(), Box::new(EvalLayout::VConcat(Box::new(a), Box::new(b)))))
                    .unwrap_or(EvalLayout::Empty)
            },
            Self::HLine(symbol) => {
                EvalLayout::HLine(*symbol)
            },
            Self::VLine(symbol) => {
                EvalLayout::VLine(*symbol)
            },
            Self::Text(s) => {
                let space = s.iter().map(|c| c.glyph.width().unwrap_or(0)).sum();
                EvalLayout::Cached(HashMap::new(), Box::new(EvalLayout::Text(s.clone().into(), space)))
            },
            Self::ExactWidth(layout, width) => {
                EvalLayout::ExactWidth(Box::new(layout.to_eval()), *width)
            },
            Self::ExactHeight(layout, height) => {
                EvalLayout::ExactHeight(Box::new(layout.to_eval()), *height)
            },
            Self::Weight(layout, factor) => {
                EvalLayout::Weight(Box::new(layout.to_eval()), *factor)
            },
            Self::Diminish(layout) => {
                EvalLayout::Diminish(Box::new(layout.to_eval()))
            },
        }
    }

    pub fn display<W: Write>(&self, pos: Pos, mut size: Size, term: &mut Terminal<W>) {
        let mut e = self.to_eval();
        let (mut layout, score) = e.eval(size).unwrap();
        while size.height >= 1 {
            size.height -= 1;
            if let Some((small_layout, small_score)) = e.eval(size) {
                if small_score == score {
                    layout = small_layout;
                }
            }
        }
        layout.display(pos, term)
    }
}

impl EvalLayout {
    fn exact_width(&self) -> Option<usize> {
        match self {
            Self::HConcat(a, b) => {
                Some(a.exact_width()? + b.exact_width()?)
            },
            Self::VConcat(a, b) => {
                a.exact_width().or(b.exact_width())
            },
            Self::VLine(_) => {
                Some(1)
            },
            Self::ExactWidth(_, width) => {
                Some(*width)
            },
            Self::Weight(a, _) => {
                a.exact_width()
            },
            Self::Diminish(a) => {
                a.exact_width()
            },
            Self::Cached(_, a) => {
                a.exact_width()
            },
            _ => {
                None
            },
        }
    }

    fn exact_height(&self) -> Option<usize> {
        match self {
            Self::HConcat(a, b) => {
                a.exact_height().or(b.exact_height())
            },
            Self::VConcat(a, b) => {
                Some(a.exact_height()? + b.exact_height()?)
            },
            Self::HLine(_) => {
                Some(1)
            },
            Self::ExactHeight(_, height) => {
                Some(*height)
            },
            Self::Weight(a, _) => {
                a.exact_height()
            },
            Self::Diminish(a) => {
                a.exact_height()
            },
            Self::Cached(_, a) => {
                a.exact_height()
            },
            _ => {
                None
            },
        }
    }

    fn eval(&mut self, size: Size) -> Option<(SizedLayout, f64)> {
        match self {
            Self::Empty => {
                Some((SizedLayout::Empty(size), 0f64))
            },
            Self::HConcat(a, b) => {
                let a_width_range =
                    match (a.exact_width(), b.exact_width()) {
                        (Some(a_width), Some(b_width)) => {
                            if a_width + b_width != size.width {
                                return None;
                            }
                            a_width ..= a_width
                        },
                        (Some(a_width), _) => {
                            if a_width > size.width {
                                return None;
                            }
                            a_width ..= a_width
                        },
                        (_, Some(b_width)) => {
                            if b_width > size.width {
                                return None;
                            }
                            let a_width = size.width - b_width;
                            a_width ..= a_width
                        },
                        _ => {
                            0 ..= size.width
                        },
                    };
                a_width_range.filter_map(|a_width| {
                        let a_size = Size { width: a_width, .. size };
                        let b_size = Size { width: size.width - a_width, .. size };
                        let (a_layout, a_score) = a.eval(a_size)?;
                        let (b_layout, b_score) = b.eval(b_size)?;
                        Some((SizedLayout::HConcat(vec![a_layout, b_layout].into()), a_score + b_score))
                    }).max_by(|(_, x), (_, y)| {
                        x.partial_cmp(y).expect("NaN should not occur")
                    })
            },
            Self::VConcat(a, b) => {
                let a_height_range =
                    match (a.exact_height(), b.exact_height()) {
                        (Some(a_height), Some(b_height)) => {
                            if a_height + b_height != size.height {
                                return None;
                            }
                            a_height ..= a_height
                        },
                        (Some(a_height), _) => {
                            if a_height > size.height {
                                return None;
                            }
                            a_height ..= a_height
                        },
                        (_, Some(b_height)) => {
                            if b_height > size.height {
                                return None;
                            }
                            let a_height = size.height - b_height;
                            a_height ..= a_height
                        },
                        _ => {
                            0 ..= size.height
                        },
                    };
                a_height_range.filter_map(|a_height| {
                        let a_size = Size { height: a_height, .. size };
                        let b_size = Size { height: size.height - a_height, .. size };
                        let (a_layout, a_score) = a.eval(a_size)?;
                        let (b_layout, b_score) = b.eval(b_size)?;
                        Some((SizedLayout::VConcat(vec![a_layout, b_layout].into()), a_score + b_score))
                    }).max_by(|(_, x), (_, y)| {
                        x.partial_cmp(y).expect("NaN should not occur")
                    })
            },
            Self::HLine(symbol) => {
                let score = if size.height == 0 {
                    0f64
                } else {
                    1000f64 - size.height as f64
                };
                Some((SizedLayout::Fill(*symbol, size), score))
            },
            Self::VLine(symbol) => {
                let score = if size.width == 0 {
                    0f64
                } else {
                    1000f64 - size.width as f64
                };
                Some((SizedLayout::Fill(*symbol, size), score))
            },
            Self::Text(symbols, space) => {
                let avail_space = size.width * size.height;
                Some((SizedLayout::Text(symbols.clone(), size), avail_space.min(*space) as f64))
            },
            Self::ExactWidth(a, _) => {
                a.eval(size)
            },
            Self::ExactHeight(a, _) => {
                a.eval(size)
            },
            Self::Weight(a, factor) => {
                let (layout, score) = a.eval(size)?;
                Some((layout, score * *factor))
            },
            Self::Diminish(a) => {
                let (layout, score) = a.eval(size)?;
                Some((layout, score.sqrt()))
            },
            Self::Cached(cache, a) => {
                cache.entry(size).or_insert_with(|| a.eval(size)).clone()
            },
        }
    }
}

impl SizedLayout {
    fn size(&self) -> Size {
        match self {
            Self::Empty(size) => {
                *size
            },
            Self::HConcat(layouts) => {
                let mut size = Size::null();
                for layout in layouts.iter() {
                    let subsize = layout.size();
                    size.width += subsize.width;
                    size.height = size.height.max(subsize.height);
                }
                size
            },
            Self::VConcat(layouts) => {
                let mut size = Size::null();
                for layout in layouts.iter() {
                    let subsize = layout.size();
                    size.width = size.width.max(subsize.width);
                    size.height += subsize.height;
                }
                size
            },
            Self::Fill(_, size) => {
                *size
            },
            Self::Text(_, size) => {
                *size
            },
        }
    }

    fn display<W: Write>(&self, pos: Pos, term: &mut Terminal<W>) {
        match self {
            Self::Empty(_) => {},
            Self::HConcat(layouts) => {
                let mut pos = pos;
                for layout in layouts.iter() {
                    layout.display(pos, term);
                    pos.x += layout.size().width;
                }
            },
            Self::VConcat(layouts) => {
                let mut pos = pos;
                for layout in layouts.iter() {
                    layout.display(pos, term);
                    pos.y += layout.size().height;
                }
            },
            Self::Fill(symbol, size) => {
                term.batch(Action::SetForegroundColor(symbol.foreground)).unwrap();
                term.batch(Action::SetBackgroundColor(symbol.background)).unwrap();
                for y in pos.y .. pos.y + size.height {
                    term.batch(Action::MoveCursorTo(pos.x as u16, y as u16)).unwrap();
                    for _ in 0 .. size.width {
                        write!(term, "{}", symbol.glyph).unwrap();
                    }
                }
            },
            Self::Text(symbols, size) => {
                let mut cursor = pos;
                for symbol in symbols.iter() {
                    if let Some(advance) = symbol.glyph.width() {
                        if cursor.x + advance > pos.x + size.width {
                            cursor.x = pos.x;
                            cursor.y += 1;
                        }
                        if cursor.y >= pos.y + size.height {
                            break;
                        }
                        term.batch(Action::MoveCursorTo(cursor.x as u16, cursor.y as u16)).unwrap();
                        term.batch(Action::SetForegroundColor(symbol.foreground)).unwrap();
                        term.batch(Action::SetBackgroundColor(symbol.background)).unwrap();
                        write!(term, "{}", symbol.glyph).unwrap();
                        cursor.x += advance;
                    }
                }
                term.batch(Action::ResetColor).unwrap();
            },
        }
    }
}

impl<T: PrettyText> Pretty for T {
    fn layout(&self) -> Layout {
        let mut text = TextBuilder::new();
        self.get_text(&mut text);
        Layout::Text(text.symbols())
    }
}

impl PrettyText for &str {
    fn get_text(&self, text: &mut TextBuilder) {
        text.write_str_default(self);
    }
}

impl PrettyText for String {
    fn get_text(&self, text: &mut TextBuilder) {
        (self as &str).get_text(text);
    }
}
