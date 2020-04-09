use super::Point;

/// A rectangle in the game world.
#[derive(Clone, Copy)]
pub struct Rect {
    bl: Point,
    tr: Point,
}

impl Rect {
    /// Builds a new `Rect` given the bottom-left corner and its width and height.
    pub fn new(x: u32, y: u32, w: u32, h: u32) -> Rect {
        Rect {
            bl: [x, y].into(),
            tr: [x + w - 1, y + h - 1].into(),
        }
    }

    /// Returns the coordinate of the rectangle's left side.
    pub fn left(&self) -> u32 {
        self.bl.x()
    }

    /// Returns the coordinate of the rectangle's right side.
    pub fn right(&self) -> u32 {
        self.tr.x()
    }

    /// Returns the coordinate of the rectangle's bottom side.
    pub fn bottom(&self) -> u32 {
        self.bl.y()
    }

    /// Returns the coordinate of the rectangle's top side.
    pub fn top(&self) -> u32 {
        self.tr.y()
    }

    /// Returns the width of the rectangle.
    pub fn width(&self) -> u32 {
        self.right() - self.left() + 1
    }

    /// Returns the height of the rectangle.
    pub fn height(&self) -> u32 {
        self.top() - self.bottom() + 1
    }

    /// Returns true if `self` intersects with `other`.
    pub fn intersects(&self, other: &Rect) -> bool {
        self.left() <= other.right()
            && self.right() >= other.left()
            && self.bottom() <= other.top()
            && self.top() >= other.bottom()
    }

    /// Returns the center of this rectangle.
    pub fn center(&self) -> Point {
        Point::new(
            (self.left() + self.right()) / 2,
            (self.bottom() + self.top()) / 2,
        )
    }
}

impl From<(u32, u32, u32, u32)> for Rect {
    fn from((x, y, w, h): (u32, u32, u32, u32)) -> Self {
        Rect::new(x, y, w, h)
    }
}
