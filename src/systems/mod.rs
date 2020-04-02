pub mod ai;
pub mod map;

use crate::{
    components::{InputListener, Position, Viewshed},
    game::{RunState, TileDimension},
    input::{ActionBinding, GameBindings},
    map::WorldMap,
    math::Point,
};

use amethyst::{
    core::transform::Transform,
    derive::SystemDesc,
    ecs::{Entities, Join, Read, ReadStorage, System, SystemData, Write, WriteStorage},
    input::InputHandler,
};

use std::collections::HashSet;

#[derive(Default, SystemDesc)]
pub struct InputDispatcher {
    pressed: HashSet<ActionBinding>,
}

impl InputDispatcher {
    fn was_pressed(&self, action: &ActionBinding) -> bool {
        self.pressed.contains(action)
    }
}

impl InputDispatcher {
    fn can_move_to(&self, to: Point, map: &Read<WorldMap>) -> bool {
        !map.blocked(&to).unwrap_or(&true)
    }
}

impl<'s> System<'s> for InputDispatcher {
    type SystemData = (
        WriteStorage<'s, InputListener>,
        WriteStorage<'s, Position>,
        WriteStorage<'s, Viewshed>,
        Read<'s, InputHandler<GameBindings>>,
        Read<'s, WorldMap>,
        Write<'s, RunState>,
        Write<'s, Point>,
    );

    fn run(
        &mut self,
        (mut movers, mut positions, mut viewsheds, input, map, mut run_state, mut ppos): Self::SystemData,
    ) {
        // Keep track of the actions which are down at each update
        // to implement non-repeating key press events.
        let pressed: HashSet<_> = input
            .bindings
            .actions()
            .cloned()
            .filter(|a| input.action_is_down(a).unwrap_or_default())
            .collect();

        for (_, Position(ref mut v), Viewshed { ref mut dirty, .. }) in
            (&mut movers, &mut positions, &mut viewsheds).join()
        {
            let mut to = *v;

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

            if self.can_move_to(to, &map) && to != *v {
                *v = to;
                *ppos = *v; // update global player position
                *dirty = true; // recompute viewshed on movement
                *run_state = RunState::Running; // un-pause game logic
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
