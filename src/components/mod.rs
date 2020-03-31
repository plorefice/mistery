use crate::math::Point2;

use amethyst::ecs::{Component, DenseVecStorage};
use std::collections::HashSet;

/// Tag component for the player's entity.
#[derive(Default, Copy, Clone, Debug, Component)]
pub struct PlayerTag;

/// Logical position in the world map.
#[derive(Copy, Clone, Debug, Component)]
pub struct Position(pub Point2<u32>);

/// Component for entities that need to respond to player input.
#[derive(Default, Copy, Clone, Debug, Component)]
pub struct InputListener;

#[derive(Component)]
pub struct Viewshed {
    pub range: u32,
    pub dirty: bool,
    pub visible: HashSet<Point2<u32>>,
}

impl Viewshed {
    pub fn new(range: u32) -> Viewshed {
        Viewshed {
            range,
            dirty: true,
            visible: HashSet::new(),
        }
    }
}
