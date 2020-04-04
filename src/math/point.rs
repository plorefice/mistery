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
    pub fn x(self) -> u32 {
        self.0[0]
    }

    /// Returns the second coordinate of the point.
    pub fn y(self) -> u32 {
        self.0[1]
    }

    /// Returns a new Point shifted by `(x, y)`.
    ///
    /// # Panics
    ///
    /// Panics if shifting either coordinate results in a negative number.
    pub fn translate(self, x: i32, y: i32) -> Point {
        assert!((self.x() as i32 + x) >= 0);
        assert!((self.y() as i32 + y) >= 0);

        Point::new((self.x() as i32 + x) as u32, (self.y() as i32 + y) as u32)
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
