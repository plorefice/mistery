use crate::{
    components::{BlocksTile, Player, Position, Viewshed},
    map::{ShadowcastFoV, WorldMap},
};

use amethyst::{
    core::Hidden,
    derive::SystemDesc,
    ecs::{Entities, Join, ReadStorage, System, SystemData, Write, WriteStorage},
    renderer::SpriteRender,
};

/// System that manages and updates Viewshed components.
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
                        map.revealed_mut(pt).and_then(|rev| Some(*rev = true));
                        map.visible_mut(pt).and_then(|viz| Some(*viz = true));
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

/// System that updates blocked tiles in the world map.
#[derive(Default, SystemDesc)]
pub struct MapIndexingSystem;

impl<'s> System<'s> for MapIndexingSystem {
    type SystemData = (
        Write<'s, WorldMap>,
        ReadStorage<'s, Position>,
        ReadStorage<'s, BlocksTile>,
    );

    fn run(&mut self, (mut map, positions, blockers): Self::SystemData) {
        map.reload_blocked_tiles();
        for (pos, _) in (&positions, &blockers).join() {
            map.blocked_mut(&pos.0).and_then(|b| Some(*b = true));
        }
    }
}
