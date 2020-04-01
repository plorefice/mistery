pub mod ai;

use crate::{
    components::{InputListener, Player, Position, Viewshed},
    game::{RunState, TileDimension},
    input::{ActionBinding, GameBindings},
    map::{ShadowcastFoV, TileKind, WorldMap},
    math::Point2,
};

use amethyst::{
    core::{transform::Transform, Hidden},
    derive::SystemDesc,
    ecs::{Entities, Join, Read, ReadStorage, System, SystemData, Write, WriteStorage},
    input::InputHandler,
    renderer::SpriteRender,
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
    fn can_move_to(&self, to: Point2<u32>, map: &Read<WorldMap>) -> bool {
        map.get(to[0], to[1]) != Some(TileKind::Wall)
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
    );

    fn run(
        &mut self,
        (mut movers, mut positions, mut viewsheds, input, map, mut run_state): Self::SystemData,
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
                    ActionBinding::Up => to[1] += 1,
                    ActionBinding::Left => to[0] -= 1,
                    ActionBinding::Down => to[1] -= 1,
                    ActionBinding::Right => to[0] += 1,
                }
            }

            if self.can_move_to(to, &map) && to != *v {
                *v = to;
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

#[derive(Default, SystemDesc)]
pub struct VisibilitySystem;

impl<'s> System<'s> for VisibilitySystem {
    type SystemData = (
        Entities<'s>,
        ReadStorage<'s, Player>,
        ReadStorage<'s, Position>,
        ReadStorage<'s, SpriteRender>,
        WriteStorage<'s, Viewshed>,
        WriteStorage<'s, Hidden>,
        Write<'s, WorldMap>,
    );

    fn run(
        &mut self,
        (entities, players, positions, renders, mut viewsheds, mut hiddens, mut map): Self::SystemData,
    ) {
        for (player, &Position(pos), vs) in (&entities, &positions, &mut viewsheds).join() {
            if vs.dirty {
                vs.visible = ShadowcastFoV::run(&*map, pos[0], pos[1], vs.range);
                vs.dirty = false;

                // If the entity is also a player, perform some additional actions
                if players.contains(player) {
                    // First, reveal the visible tiles on the map
                    map.clear_visibility();
                    for pt in &vs.visible {
                        map.reveal(pt[0], pt[1]);
                        map.set_visible(pt[0], pt[1], true);
                    }

                    // For renderable entities, hide those that are not in view
                    // and show those that are visible
                    for (e, _, &Position(other)) in (&entities, &renders, &positions).join() {
                        if e != player {
                            if vs.visible.contains(&other) {
                                hiddens.remove(e);
                            } else {
                                hiddens.insert(e, Hidden).unwrap();
                            }
                        }
                    }
                }
            }
        }
    }
}
