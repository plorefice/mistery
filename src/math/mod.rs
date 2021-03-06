//! This module contains math utilities such as geometry suff, distance algorithms, etc.

mod point;
mod rect;

// Re-export all modules
pub use point::*;
pub use rect::*;

/// Pythagorean distance between two points, rounded down to the nearest integer.
pub fn distance_2d(p1: Point, p2: Point) -> u32 {
    let dx2 = (p2.x() as i32 - p1.x() as i32).pow(2);
    let dy2 = (p2.y() as i32 - p1.y() as i32).pow(2);
    f32::sqrt((dx2 + dy2) as f32).floor() as u32
}
