//! This module contains all the components of the ECS used throughout the game.

use crate::math::Point;

use amethyst::ecs::{Component, DenseVecStorage, Entity, WriteStorage};
use std::collections::HashSet;

/// Tag component for the player's entity.
#[derive(Component)]
pub struct Player;

/// Tag component for entities that can act in a turn.
#[derive(Default, Copy, Clone, Component)]
pub struct ActsOnTurns {
    ap: u32,
}

impl ActsOnTurns {
    pub fn can_act(self) -> bool {
        self.ap > 0
    }

    pub fn refresh(&mut self) {
        self.ap = 1;
    }

    pub fn perform(&mut self) -> bool {
        if self.can_act() {
            self.ap -= 1;
            true
        } else {
            false
        }
    }
}

/// Tag component for an entity belonging to a faction.
#[derive(Component, PartialEq)]
pub struct Faction(pub u32);

/// Tag component for entities that can be picked up from the ground.
#[derive(Component)]
pub struct Pickable;

/// Component for named entities.
#[derive(Component)]
pub struct Name(pub String);

/// Logical position in the world map.
#[derive(Component)]
pub struct Position(pub Point);

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
#[derive(Component)]
pub struct BlocksTile;

/// Component for entities that can heal the user for a certain amount.
#[derive(Component)]
pub struct HealsUser {
    pub amount: i32,
}

/// Component for entities (usually `Item`s) located in another's entity inventory.
#[derive(Component)]
pub struct InBackpack {
    pub owner: Entity,
}

/// Component for entities that can participate in a fight.
#[derive(Component)]
pub struct CombatStats {
    pub hp: i32,
    pub max_hp: i32,
    pub defense: i32,
    pub power: i32,
}

/// Component for entities that have decided to move in their turn.
#[derive(Component)]
pub struct WantsToMove {
    pub to: Point,
}

/// Component for entities that want to pick up a `Pickable` entity.
#[derive(Component)]
pub struct WantsToPickUp {
    pub what: Entity,
}

/// Component for entities that have decided to use an item.
#[derive(Component)]
pub struct WantsToUseItem {
    pub what: Entity,
}

/// Component for entities that have decided to drop an item.
#[derive(Component)]
pub struct WantsToDropItem {
    pub what: Entity,
}

/// Component for entities that are being targeted by another entity for melee combat.
#[derive(Default, Component)]
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

/// Component for entities that have to suffer an amout of damage.
#[derive(Default, Component)]
pub struct SuffersDamage {
    pub damage: u32,
}

impl SuffersDamage {
    /// Adds some damage to the total suffered by an entity.
    pub fn damage(store: &mut WriteStorage<SuffersDamage>, who: Entity, amount: u32) {
        store
            .entry(who)
            .unwrap()
            .or_insert(SuffersDamage::default())
            .damage += amount;
    }
}
