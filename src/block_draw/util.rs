use std::{ops::{Add, AddAssign, Sub, SubAssign, Mul, MulAssign}, iter::Peekable};

use petgraph::stable_graph::NodeIndex;
use svg::node::element::path::Parameters;

#[derive(Clone,Copy)]
struct Rgb(u8, u8, u8);

#[derive(Clone, Copy, PartialEq, Default, Debug)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub const ZERO: Vec2 = Vec2::new(0.0, 0.0);
    pub const Q1: Vec2 = Vec2::new(1.0, 1.0);
    pub const Q2: Vec2 = Vec2::new(-1.0, 1.0);
    pub const Q3: Vec2 = Vec2::new(-1.0, -1.0);
    pub const Q4: Vec2 = Vec2::new(1.0, -1.0);

    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub const fn squared(n: f32) -> Self {
        Self { x: n, y: n }
    }
    pub const fn x(x: f32) -> Self {
        Self { x, y: 0.0 }
    }

    pub const fn y(y: f32) -> Self {
        Self { x: 0.0, y }
    }

    pub fn min(&self, other: Self) -> Self {
        Self { x: self.x.min(other.x), y: self.y.min(other.y) }
    }

    pub fn max(&self, other: Self) -> Self {
        Self { x: self.x.max(other.x), y: self.y.max(other.y) }
    }
}

impl Add for Vec2 {
    type Output = Vec2;

    fn add(self, rhs: Self) -> Self::Output {
        Self { x: self.x + rhs.x, y: self.y + rhs.y }
    }
}

impl AddAssign for Vec2 {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl Mul for Vec2 {
    type Output = Vec2;

    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
        }
    }
}

impl MulAssign for Vec2 {
    fn mul_assign(&mut self, rhs: Self) {
        self.x *= rhs.x;
        self.y *= rhs.y;
    }
}

impl<N: Clone> Mul<N> for Vec2
where f32: Mul<N, Output=f32> {
    type Output = Vec2;

    fn mul(self, rhs: N) -> Self::Output {
        Self {
            x: self.x * rhs.clone(),
            y: self.y * rhs,
        }
    }
}

impl<N: Clone> MulAssign<N> for Vec2
where f32: MulAssign<N> {
    fn mul_assign(&mut self, rhs: N) {
        self.x *= rhs.clone();
        self.y *= rhs;
    }
}

impl Sub for Vec2 {
    type Output = Vec2;

    fn sub(self, rhs: Self) -> Self::Output {
        Self { x: self.x - rhs.x, y: self.y - rhs.y }
    }
}


impl SubAssign for Vec2 {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl From<Vec2> for svg::node::element::path::Parameters {
    fn from(Vec2 { x, y }: Vec2) -> Self {
        Parameters::from((x, y))
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bounds {
    pub min: Vec2,
    pub max: Vec2,
}

impl Bounds {
    pub fn closed_at(position: Vec2) -> Self {
        Self { min: position, max: position }
    }

    pub fn svg_viewbox(&self) -> (f32, f32, f32, f32) {
        let Vec2 { x, y } = self.min;
        let Vec2 { x: width, y: height } = self.max - self.min;
        (x, y, width, height)
    }

    pub fn expand(&mut self, other: Self) {
        self.min = self.min.min(other.min);
        self.max = self.max.max(other.max);
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
            _ => return None,
        };

        loop {
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