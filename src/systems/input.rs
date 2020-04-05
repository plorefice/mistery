//! This module contains all the input-related systems.

use crate::components::{ActsOnTurns, InputListener, Position, WantsToMove};

use amethyst::{
    derive::SystemDesc,
    ecs::{Entities, Join, Read, ReadStorage, System, SystemData, WriteStorage},
    input::{BindingTypes, InputHandler},
};
use serde::{Deserialize, Serialize};
use std::fmt;

use std::collections::HashSet;

/// Stub implementation for axis bindings. Not actually used right now.
#[derive(Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct AxisBindings;

impl fmt::Display for AxisBindings {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Axis bindings")
    }
}

/// Custom implementation for action bindings.
#[derive(Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActionBinding {
    MoveN,
    MoveW,
    MoveS,
    MoveE,
    MoveNW,
    MoveSW,
    MoveSE,
    MoveNE,
    PickUp,
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
        WriteStorage<'s, ActsOnTurns>,
        WriteStorage<'s, WantsToMove>,
        Read<'s, InputHandler<GameBindings>>,
    );

    fn run(
        &mut self,
        (entities, listeners, positions, mut actors, mut movers, input): Self::SystemData,
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

            let mut to = p;

            for action in self.active_actions() {
                match action {
                    ActionBinding::MoveN => to[1] += 1,
                    ActionBinding::MoveW => to[0] -= 1,
                    ActionBinding::MoveS => to[1] -= 1,
                    ActionBinding::MoveE => to[0] += 1,
                    ActionBinding::MoveNW => {
                        to[1] += 1;
                        to[0] -= 1;
                    }
                    ActionBinding::MoveSW => {
                        to[1] -= 1;
                        to[0] -= 1;
                    }
                    ActionBinding::MoveSE => {
                        to[1] -= 1;
                        to[0] += 1;
                    }
                    ActionBinding::MoveNE => {
                        to[1] += 1;
                        to[0] += 1;
                    }
                    ActionBinding::PickUp => {}
                }
            }

            if to != p {
                movers.insert(e, WantsToMove { to }).unwrap();
            }
        }
    }
}
