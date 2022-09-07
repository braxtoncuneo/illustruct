use std::{ops::Add, iter::Peekable};

use petgraph::stable_graph::NodeIndex;

#[derive(Clone,Copy)]
struct RGB(u8, u8, u8);

#[derive(Clone, Copy, PartialEq, Default)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

impl Add for Vec2 {
    type Output = Vec2;

    fn add(self, rhs: Self) -> Self::Output {
        Self { x: self.x + rhs.x, y: self.y + rhs.y }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct BlockAdjSpan {
    pub min: f32,
    pub max: f32,
    pub index: NodeIndex,
}

pub struct BlockAdjListPairIter<A: Iterator, B: Iterator>{
    a: Peekable<A>,
    b: Peekable<B>,
}

impl<A: Iterator, B: Iterator> BlockAdjListPairIter<A, B> {
    pub fn new(a: impl IntoIterator<IntoIter=A>, b: impl IntoIterator<IntoIter=B>) -> Self {
        Self {
            a: a.into_iter().peekable(),
            b: b.into_iter().peekable(),
        }
    }
}

impl<A, B> Iterator for BlockAdjListPairIter <A, B>
where
    A: Iterator<Item=BlockAdjSpan>,
    B: Iterator<Item=BlockAdjSpan>,
{
    type Item = (NodeIndex, NodeIndex);

    fn next(&mut self) -> Option<Self::Item> {
        let (a, b) = match (self.a.peek().copied(), self.b.peek().copied()) {
            (Some(x), Some(y)) => (x, y),
            (x, y) => {
                println!("x is_some = {} y is_some = {}", x.is_some(), y.is_some());
                return None;
            }
        };

        loop {
            dbg!((a, b));
            if a.max < b.min {
                self.a.next();
            } else if b.max < a.min {
                self.b.next();
            } else if a.max < b.max {
                self.a.next();
                break Some((a.index, b.index))
            } else {
                self.b.next();
                break Some((a.index, b.index))
            }
        }
    }
}

pub struct Translate(pub f32, pub f32);

impl From<Vec2> for Translate {
    fn from(Vec2 { x, y }: Vec2) -> Self {
        Self(x, y)
    }
}

impl From<Translate> for svg::node::Value {
    fn from(Translate(x, y): Translate) -> Self {
        Self::from(format!("translate({x},{y})"))
    }
}