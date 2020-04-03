use crate::math::Point;

use amethyst::ecs::{Component, DenseVecStorage, Entity, WriteStorage};
use std::collections::HashSet;

/// Tag component for the player's entity.
#[derive(Default, Copy, Clone, Debug, Component)]
pub struct Player;

/// Tag component for an entity belonging to a faction.
#[derive(Default, Copy, Clone, Debug, PartialEq, Component)]
pub struct Faction(pub u32);

/// Component for named entities.
#[derive(Default, Clone, Debug, Component)]
pub struct Name(pub String);

/// Logical position in the world map.
#[derive(Copy, Clone, Debug, Component)]
pub struct Position(pub Point);

/// Component for entities that need to respond to player input.
#[derive(Default, Copy, Clone, Debug, Component)]
pub struct InputListener;

/// Component that keeps track of a set of visible tiles in a range.
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

/// Component for entities that can participate in a fight.
#[derive(Debug, Component)]
pub struct CombatStats {
    pub hp: i32,
    pub max_hp: i32,
    pub defense: i32,
    pub power: i32,
}

/// Component for entities that have decided to move in their turn.
#[derive(Debug, Component)]
pub struct WantsToMove {
    pub to: Point,
}

/// Component for entities that are being targeted by another entity for melee combat.
#[derive(Default, Debug, Component)]
pub struct TargetedForMelee {
    pub by: Vec<Entity>,
}

impl TargetedForMelee {
    /// Targets an entity for melee combat.
    pub fn target(store: &mut WriteStorage<TargetedForMelee>, attacker: Entity, victim: Entity) {
        store
            .entry(victim)
            .unwrap()
            .or_insert(TargetedForMelee::default())
            .by
            .push(attacker);
    }
}
