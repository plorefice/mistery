//! This module contains all the input-related systems.

use crate::{
    components::{ActsOnTurns, Pickable, Player, Position, WantsToMove, WantsToPickUp},
    math::Point,
    resources::CombatLog,
    states::{GameStateWrapper, GameTrans, Intent, InventoryState},
};

use amethyst::{
    ecs::{Entities, Entity, Join, ReadStorage, World, Write, WriteStorage},
    input::BindingTypes,
    prelude::*,
};
use serde::{Deserialize, Serialize};
use std::fmt;

#[rustfmt::skip]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum Direction { N, W, S, E, NW, SW, SE, NE }

/// Stub implementation for axis bindings. Not actually used right now.
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct AxisBindings;

impl fmt::Display for AxisBindings {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Axis bindings")
    }
}

/// Custom implementation for action bindings.
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActionBinding {
    Move(Direction),
    PickUp,
    OpenInventory,
    DropItem,
    Cancel,
}

impl fmt::Display for ActionBinding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// Custom input bindings.
///
/// This replaces the stock `StringBindings` which are kinda clumsy to use.
#[derive(Default, Debug)]
pub struct GameBindings;

impl BindingTypes for GameBindings {
    type Axis = AxisBindings;
    type Action = ActionBinding;
}

/// System for input handling and dispatching in the running game state.
///
/// This is not actually a system, but I put it here because the meaning is pretty much the same.
#[derive(Default)]
pub struct RunStateInputDispatcher;

type RunStateSystemData<'s> = (
    Entities<'s>,
    ReadStorage<'s, Player>,
    ReadStorage<'s, Position>,
    ReadStorage<'s, Pickable>,
    WriteStorage<'s, ActsOnTurns>,
    WriteStorage<'s, WantsToMove>,
    WriteStorage<'s, WantsToPickUp>,
    Write<'s, CombatLog>,
);

impl RunStateInputDispatcher {
    pub fn handle(&mut self, world: &mut World, action: ActionBinding) -> GameTrans {
        let (entities, players, positions, pickables, mut actors, mut movers, mut pickers, mut log) =
            world.system_data::<RunStateSystemData>();

        if let Some((player, actor, &Position(p), _)) =
            (&entities, &mut actors, &positions, &players).join().next()
        {
            if !actor.perform() {
                return Trans::None;
            }

            match action {
                ActionBinding::Move(d) => move_player(player, p, d, &mut movers),
                ActionBinding::PickUp => pickup_item(
                    player,
                    &entities,
                    &pickables,
                    &positions,
                    &mut pickers,
                    &mut log,
                ),
                ActionBinding::OpenInventory => {
                    return Trans::Push(Box::new(GameStateWrapper::new(InventoryState::new(
                        Intent::UseItem,
                    ))));
                }
                ActionBinding::DropItem => {
                    return Trans::Push(Box::new(GameStateWrapper::new(InventoryState::new(
                        Intent::DropItem,
                    ))));
                }
                _ => (),
            }
        }

        Trans::None
    }
}

fn move_player(
    player: Entity,
    from: Point,
    dir: Direction,
    movers: &mut WriteStorage<WantsToMove>,
) {
    use Direction::*;

    let delta = match dir {
        N => (0, 1),
        W => (-1, 0),
        S => (0, -1),
        E => (1, 0),
        NW => (-1, 1),
        SW => (-1, -1),
        SE => (1, -1),
        NE => (1, 1),
    };

    movers
        .insert(player, WantsToMove { to: from + delta })
        .unwrap();
}

fn pickup_item(
    player: Entity,
    entities: &Entities,
    pickables: &ReadStorage<Pickable>,
    positions: &ReadStorage<Position>,
    pickers: &mut WriteStorage<WantsToPickUp>,
    log: &mut Write<CombatLog>,
) {
    if let Some(&Position(p)) = positions.get(player) {
        let target_item = (entities, pickables, positions)
            .join()
            .filter_map(
                |(e, _, &Position(here))| {
                    if p == here {
                        Some(e)
                    } else {
                        None
                    }
                },
            )
            .next();

        if let Some(what) = target_item {
            pickers.insert(player, WantsToPickUp { what }).unwrap();
        } else {
            log.push("There is nothing here to pick up.");
        }
    }
}
