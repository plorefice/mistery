use crate::components::*;
use crate::input::*;

use amethyst::{
    core::{transform::Transform, Time},
    derive::SystemDesc,
    ecs::{Join, Read, ReadStorage, System, SystemData, WriteStorage},
    input::InputHandler,
};

use std::collections::HashSet;

#[derive(SystemDesc)]
pub(crate) struct LeftWalker;

impl<'s> System<'s> for LeftWalker {
    type SystemData = (ReadStorage<'s, LeftMover>, WriteStorage<'s, Position>);

    fn run(&mut self, (lefty, mut positions): Self::SystemData) {
        for (_, Position(ref mut v)) in (&lefty, &mut positions).join() {
            v[0] -= 1;
            if v[0] < 0 {
                v[0] = 79
            };
        }
    }
}

#[derive(Default, SystemDesc)]
pub(crate) struct InputMovementSystem {
    pressed: HashSet<ActionBinding>,
}

impl InputMovementSystem {
    fn was_pressed(&self, action: &ActionBinding) -> bool {
        self.pressed.contains(action)
    }
}

impl<'s> System<'s> for InputMovementSystem {
    type SystemData = (
        WriteStorage<'s, InputMover>,
        WriteStorage<'s, Position>,
        Read<'s, InputHandler<GameBindings>>,
        Read<'s, Time>,
    );

    fn run(&mut self, (mut movers, mut positions, input, _time): Self::SystemData) {
        // Keep track of the actions which are down at each update
        // to implement non-repeating key press events.
        let pressed: HashSet<_> = input
            .bindings
            .actions()
            .cloned()
            .filter(|a| input.action_is_down(a).unwrap_or_default())
            .collect();

        for (_, Position(ref mut v)) in (&mut movers, &mut positions).join() {
            for action in pressed.iter().filter(|a| !self.was_pressed(a)) {
                match action {
                    ActionBinding::Up => v[1] += 1,
                    ActionBinding::Left => v[0] -= 1,
                    ActionBinding::Down => v[1] -= 1,
                    ActionBinding::Right => v[0] += 1,
                }
            }

            v[0] = v[0].max(0).min(79);
            v[1] = v[1].max(0).min(49);
        }

        // Store currently active actions
        self.pressed = pressed;
    }
}

#[derive(Default, SystemDesc)]
pub(crate) struct PositionTranslator;

impl<'s> System<'s> for PositionTranslator {
    type SystemData = (ReadStorage<'s, Position>, WriteStorage<'s, Transform>);

    fn run(&mut self, (positions, mut transforms): Self::SystemData) {
        for (pos, trans) in (&positions, &mut transforms).join() {
            trans.set_translation_xyz(
                pos.0[0] as f32 * 20.0 + 10.0,
                pos.0[1] as f32 * 20.0 + 10.0,
                0.0,
            );
        }
    }
}
