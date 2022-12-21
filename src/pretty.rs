use std::io::Write;
use terminal::Terminal;

#[derive(Debug, Clone, Copy)]
pub struct Pos {
    pub x: usize,
    pub y: usize,
}

#[derive(Debug, Clone, Copy)]
pub struct Size {
    pub width: usize,
    pub height: usize,
}

#[derive(Debug, Clone, Copy)]
pub struct Rect {
    pub pos: Pos,
    pub size: Size,
}

#[derive(Debug, Clone, Copy)]
pub struct Req {
    pub min_space: usize,
    pub wanted_space: usize,
    pub min_size: Size,
    pub wanted_size: Size,
}

pub trait Pretty {
    fn requirements(&self) -> Req;
    fn pretty<W: Write>(&self, region: Rect, term: &mut Terminal<W>);
}
