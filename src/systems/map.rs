use crate::{
    components::{BlocksTile, Player, Position, Viewshed, WantsToMove},
    map::{ShadowcastFoV, WorldMap},
    math::Point,
};

use amethyst::{
    core::Hidden,
    derive::SystemDesc,
    ecs::{Entities, Entity, Join, ReadStorage, System, SystemData, Write, WriteStorage},
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

/// System that manages entities that want to move in this turn.
#[derive(Default, SystemDesc)]
pub struct MoveResolver;

impl MoveResolver {
    fn move_entity(
        &self,
        _: Entity,
        map: &mut WorldMap,
        from: &mut Point,
        to: Point,
        blocks: bool,
    ) {
        // Move the blocked tile, if the entity is blocking
        if blocks {
            map.blocked_mut(from).and_then(|b| Some(*b = false));
            map.blocked_mut(&to).and_then(|b| Some(*b = true));
        }

        *from = to;
    }
}

impl<'s> System<'s> for MoveResolver {
    type SystemData = (
        Entities<'s>,
        ReadStorage<'s, Player>,
        ReadStorage<'s, BlocksTile>,
        WriteStorage<'s, Position>,
        WriteStorage<'s, WantsToMove>,
        WriteStorage<'s, Viewshed>,
        Write<'s, Point>,
        Write<'s, WorldMap>,
    );

    fn run(
        &mut self,
        (entitites, players, blockers, mut positions, mut movers, mut viewsheds, mut ppos, mut map): Self::SystemData,
    ) {
        for (e, Position(ref mut p), WantsToMove { to }) in
            (&entitites, &mut positions, movers.drain()).join()
        {
            if map.blocked(&to) == Some(&false) {
                self.move_entity(e, &mut map, p, to, blockers.contains(e)); // update map state

                // If the entity has a Viewshed, recompute it on movement
                if let Some(vs) = viewsheds.get_mut(e) {
                    vs.dirty = true;
                }

                // If the entity is the player, update its global position
                if players.contains(e) {
                    *ppos = to;
                }
            }
        }
    }
}
