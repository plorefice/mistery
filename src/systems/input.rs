//! This module contains all the input-related systems.

use crate::{
    components::{ActsOnTurns, InputListener, Position, WantsToMove},
    input::{ActionBinding, GameBindings},
};

use amethyst::{
    derive::SystemDesc,
    ecs::{Entities, Join, Read, ReadStorage, System, SystemData, WriteStorage},
    input::InputHandler,
};

use std::collections::HashSet;

/// System for input handling and dispatching.
///
/// The system is used to dispatch user input to the player entity.
#[derive(Default, SystemDesc)]
pub struct InputDispatcher {
    pressed: HashSet<ActionBinding>,
}

impl InputDispatcher {
    fn was_pressed(&self, action: &ActionBinding) -> bool {
        self.pressed.contains(action)
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
        let pressed: HashSet<_> = input
            .bindings
            .actions()
            .cloned()
            .filter(|a| input.action_is_down(a).unwrap_or_default())
            .collect();

        for (e, _, &Position(p), actor) in (&entities, &listeners, &positions, &mut actors).join() {
            let mut to = p;

            for action in pressed.iter().filter(|a| !self.was_pressed(a)) {
                match action {
                    ActionBinding::N => to[1] += 1,
                    ActionBinding::W => to[0] -= 1,
                    ActionBinding::S => to[1] -= 1,
                    ActionBinding::E => to[0] += 1,
                    ActionBinding::NW => {
                        to[1] += 1;
                        to[0] -= 1;
                    }
                    ActionBinding::SW => {
                        to[1] -= 1;
                        to[0] -= 1;
                    }
                    ActionBinding::SE => {
                        to[1] -= 1;
                        to[0] += 1;
                    }
                    ActionBinding::NE => {
                        to[1] += 1;
                        to[0] += 1;
                    }
                }
            }

            if to != p && actor.perform() {
                movers.insert(e, WantsToMove { to }).unwrap();
            }
        }

        // Store currently active actions
        self.pressed = pressed;
    }
}
