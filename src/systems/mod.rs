pub mod ai;
pub mod combat;
pub mod map;

use crate::{
    components::{ActsOnTurns, InputListener, Player, Position, WantsToMove},
    game::TileDimension,
    input::{ActionBinding, GameBindings},
};

use amethyst::{
    core::transform::Transform,
    derive::SystemDesc,
    ecs::{Entities, Join, Read, ReadStorage, System, SystemData, WriteStorage},
    input::InputHandler,
};

use std::collections::HashSet;

/// Enum representing one of the possible turns in the state logic.
#[derive(Copy, Clone, PartialEq)]
pub enum Turn {
    Player,
    Others,
}

impl Default for Turn {
    fn default() -> Self {
        Turn::Player
    }
}

/// System that manages which entities get to act in the current turn.
#[derive(Default, SystemDesc)]
pub struct TurnSystem {
    current: Turn,
}

impl TurnSystem {}

impl<'s> System<'s> for TurnSystem {
    type SystemData = (WriteStorage<'s, ActsOnTurns>, ReadStorage<'s, Player>);

    fn run(&mut self, (mut actors, players): Self::SystemData) {
        match self.current {
            Turn::Player => {
                if (&actors, &players).join().any(|(a, _)| a.can_act()) {
                    return;
                }
                for (actor, _) in (&mut actors, !&players).join() {
                    actor.refresh();
                }
                self.current = Turn::Others;
            }
            Turn::Others => {
                if (&actors, !&players).join().any(|(a, _)| a.can_act()) {
                    return;
                }
                for (actor, _) in (&mut actors, &players).join() {
                    actor.refresh();
                }
                self.current = Turn::Player;
            }
        }
    }
}

/// System for input handling and dispatching.
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

/// Converts a `Position` component into a `Transform` component.
///
/// The default renderer works with pixels, while for this kind of game logic
/// it's easier to reason in terms of integer tile indexes.
/// This system takes this logical representation of position and turns it into
/// something that the rendering system can actually work with.
#[derive(Default, SystemDesc)]
pub struct PositionTranslator;

impl<'s> System<'s> for PositionTranslator {
    type SystemData = (
        Entities<'s>,
        ReadStorage<'s, Position>,
        WriteStorage<'s, Transform>,
        Read<'s, TileDimension>,
    );

    fn run(&mut self, (entities, positions, mut transforms, tile_dimension): Self::SystemData) {
        let mul = tile_dimension.0 as f32;

        for (e, pos) in (&*entities, &positions).join() {
            if let Some(t) = transforms.get_mut(e) {
                t.set_translation_xyz(pos.0[0] as f32 * mul, pos.0[1] as f32 * mul, 0.0);
            } else {
                // If the entity was created without a Transform component,
                // add one the first time the system is run.
                let mut t = Transform::default();
                t.set_translation_xyz(pos.0[0] as f32 * mul, pos.0[1] as f32 * mul, 0.0);
                transforms.insert(e, t).expect("inserting transform failed");
            }
        }
    }
}
