//! This module contains all the input-related systems.

use crate::{
    components::{ActsOnTurns, InputListener, Pickable, Position, WantsToMove, WantsToPickUp},
    math::Point,
    resources::CombatLog,
};

use amethyst::{
    derive::SystemDesc,
    ecs::{Entities, Entity, Join, Read, ReadStorage, System, SystemData, Write, WriteStorage},
    input::{BindingTypes, InputHandler},
};
use serde::{Deserialize, Serialize};

use std::{collections::HashSet, fmt};

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

/// System for input handling and dispatching.
///
/// The system is used to dispatch user input to the player entity.
#[derive(Default, SystemDesc)]
pub struct InputDispatcher {
    previous: HashSet<ActionBinding>,
    current: HashSet<ActionBinding>,
}

impl InputDispatcher {
    fn update_actions(&mut self, actions: HashSet<ActionBinding>) {
        std::mem::swap(&mut self.previous, &mut self.current);
        self.current = actions;
    }

    fn active_actions(&self) -> impl Iterator<Item = &ActionBinding> {
        self.current.difference(&self.previous)
    }
}

impl<'s> System<'s> for InputDispatcher {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        Entities<'s>,
        ReadStorage<'s, InputListener>,
        ReadStorage<'s, Position>,
        ReadStorage<'s, Pickable>,
        WriteStorage<'s, ActsOnTurns>,
        WriteStorage<'s, WantsToMove>,
        WriteStorage<'s, WantsToPickUp>,
        Read<'s, InputHandler<GameBindings>>,
        Write<'s, CombatLog>,
    );

    fn run(
        &mut self,
        (
            entities,
            listeners,
            positions,
            pickables,
            mut actors,
            mut movers,
            mut pickers,
            input,
            mut log,
        ): Self::SystemData,
    ) {
        // Keep track of the actions which are down at each update
        // to implement non-repeating key press events.
        self.update_actions(
            input
                .bindings
                .actions()
                .filter(|a| input.action_is_down(a).unwrap_or_default())
                .cloned()
                .collect(),
        );

        // Do nothing if no action is active
        if self.active_actions().count() == 0 {
            return;
        }

        // This should actually loop only once right now
        for (e, _, &Position(p), actor) in (&entities, &listeners, &positions, &mut actors).join() {
            if !actor.perform() {
                continue;
            }

            for action in self.active_actions() {
                match action {
                    ActionBinding::Move(d) => move_entity(e, p, *d, &mut movers),
                    ActionBinding::PickUp => {
                        let target_item = (&entities, &pickables, &positions)
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
                            pickers.insert(e, WantsToPickUp { what }).unwrap();
                        } else {
                            log.push("There is nothing here to pick up.");
                        }
                    }
                    _ => (),
                }
            }
        }
    }
}

fn move_entity(e: Entity, from: Point, dir: Direction, movers: &mut WriteStorage<WantsToMove>) {
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

    movers.insert(e, WantsToMove { to: from + delta }).unwrap();
}
