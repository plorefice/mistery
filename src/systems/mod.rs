use crate::components::*;
use crate::game::*;
use crate::input::*;

use amethyst::{
    core::{math::Vector2, transform::Transform},
    derive::SystemDesc,
    ecs::{Entities, Join, Read, ReadStorage, System, SystemData, WriteStorage},
    input::InputHandler,
};

use std::collections::HashSet;

#[derive(Default, SystemDesc)]
pub(crate) struct InputDispatcher {
    pressed: HashSet<ActionBinding>,
}

impl InputDispatcher {
    fn was_pressed(&self, action: &ActionBinding) -> bool {
        self.pressed.contains(action)
    }
}

impl InputDispatcher {
    fn try_move(&self, pos: &mut Vector2<i32>, dx: i32, dy: i32, map: &Read<WorldMap>) {
        let try_pos = Vector2::new(pos[0] + dx, pos[1] + dy);
        if map.get((pos[0] + dx) as u32, (pos[1] + dy) as u32) != Some(TileKind::Wall) {
            *pos = try_pos;
        }
    }
}

impl<'s> System<'s> for InputDispatcher {
    type SystemData = (
        WriteStorage<'s, InputListener>,
        WriteStorage<'s, Position>,
        Read<'s, InputHandler<GameBindings>>,
        Read<'s, WorldMap>,
    );

    fn run(&mut self, (mut movers, mut positions, input, map): Self::SystemData) {
        // Keep track of the actions which are down at each update
        // to implement non-repeating key press events.
        let pressed: HashSet<_> = input
            .bindings
            .actions()
            .cloned()
            .filter(|a| input.action_is_down(a).unwrap_or_default())
            .collect();

        for (_, Position(ref mut v)) in (&mut movers, &mut positions).join() {
            let (mut dx, mut dy) = (0, 0);
            for action in pressed.iter().filter(|a| !self.was_pressed(a)) {
                match action {
                    ActionBinding::Up => dy = 1,
                    ActionBinding::Left => dx = -1,
                    ActionBinding::Down => dy = -1,
                    ActionBinding::Right => dx = 1,
                }
            }
            self.try_move(v, dx, dy, &map);
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
pub(crate) struct PositionTranslator;

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
