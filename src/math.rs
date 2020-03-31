pub use amethyst::core::math::Point2;

#[derive(Clone, Copy)]
pub struct Rect {
    bl: Point2<u32>,
    tr: Point2<u32>,
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
        self.bl[0]
    }

    /// Returns the coordinate of the rectangle's right side.
    pub fn right(&self) -> u32 {
        self.tr[0]
    }

    /// Returns the coordinate of the rectangle's bottom side.
    pub fn bottom(&self) -> u32 {
        self.bl[1]
    }

    /// Returns the coordinate of the rectangle's top side.
    pub fn top(&self) -> u32 {
        self.tr[1]
    }

    /// Returns true if `self` intersects with `other`.
    pub fn intersects(&self, other: &Rect) -> bool {
        self.left() <= other.right()
            && self.right() >= other.left()
            && self.bottom() <= other.top()
            && self.top() >= other.bottom()
    }

    /// Returns the center of this rectangle.
    pub fn center(&self) -> Point2<u32> {
        Point2::new(
            (self.left() + self.right()) / 2,
            (self.bottom() + self.top()) / 2,
        )
    }
}
