use crate::math::Point;

use amethyst::ecs::{Component, DenseVecStorage};
use std::collections::HashSet;

/// Tag component for the player's entity.
#[derive(Default, Copy, Clone, Debug, Component)]
pub struct Player;

#[derive(Default, Copy, Clone, Debug, Component)]
/// Tag component for a monster's entity.
pub struct Monster;

/// Component for named entities.
#[derive(Default, Clone, Debug, Component)]
pub struct Name(pub String);

/// Logical position in the world map.
#[derive(Copy, Clone, Debug, Component)]
pub struct Position(pub Point);

/// Component for entities that need to respond to player input.
#[derive(Default, Copy, Clone, Debug, Component)]
pub struct InputListener;

#[derive(Component)]
pub struct Viewshed {
    pub range: u32,
    pub dirty: bool,
    pub visible: HashSet<Point>,
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

/// Component for entities that block their tile in the world map.
#[derive(Default, Copy, Clone, Debug, Component)]
pub struct BlocksTile;
