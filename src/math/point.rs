use amethyst::core::math::Point2;

use std::ops::{Index, IndexMut};

/// 2D point in the game world.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Point(Point2<u32>);

impl Default for Point {
    fn default() -> Self {
        Point::new(0, 0)
    }
}

impl Point {
    /// Creates a new point.
    pub fn new(x: u32, y: u32) -> Point {
        Point(Point2::new(x, y))
    }

    /// Returns the first coordinate of the point.
    pub fn x(&self) -> u32 {
        self.0[0]
    }

    /// Returns the second coordinate of the point.
    pub fn y(&self) -> u32 {
        self.0[1]
    }
}

impl From<[u32; 2]> for Point {
    fn from(p: [u32; 2]) -> Self {
        Point::new(p[0], p[1])
    }
}

impl From<(u32, u32)> for Point {
    fn from(p: (u32, u32)) -> Self {
        Point::new(p.0, p.1)
    }
}

impl Index<usize> for Point {
    type Output = u32;

    fn index(&self, index: usize) -> &Self::Output {
        self.0.index(index)
    }
}

impl IndexMut<usize> for Point {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.0.index_mut(index)
    }
}
