//! This module contains all the map-related systems.

use crate::{
    components::*,
    core::map::{ShadowcastFoV, WorldMap},
    math::Point,
    resources::TileDimension,
};

use amethyst::{
    core::{transform::Transform, Hidden},
    derive::SystemDesc,
    ecs::{Entities, Entity, Join, Read, ReadStorage, System, SystemData, Write, WriteStorage},
    renderer::SpriteRender,
};

/// Refreshes the map's internal index.
///
/// Mainly used for recomputing the map's blocked tiles once a unit dies.
#[derive(SystemDesc)]
pub struct MapIndexingSystem;

impl<'s> System<'s> for MapIndexingSystem {
    type SystemData = (
        ReadStorage<'s, Position>,
        ReadStorage<'s, BlocksTile>,
        Write<'s, WorldMap>,
    );

    fn run(&mut self, (positions, blockers, mut map): Self::SystemData) {
        // Recompute blocked tiles at the end of a turn
        map.reload_blocked_tiles();
        for (_, &Position(p)) in (&blockers, &positions).join() {
            map[p].blocked = true;
        }
    }
}

/// Computes a unit's field of vision.
///
/// For each entity with a [`Viewshed`] component, this system computes their field of vision.
#[derive(SystemDesc)]
pub struct VisibilitySystem;

impl<'s> System<'s> for VisibilitySystem {
    #[allow(clippy::type_complexity)]
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
        for (e1, &Position(pos), vs) in (&entities, &positions, &mut viewsheds).join() {
            if vs.dirty {
                vs.visible = ShadowcastFoV::run(&*map, pos[0], pos[1], vs.range);
                vs.dirty = false;

                // If the entity is also a player, perform some additional actions
                if players.contains(e1) {
                    // First, reveal the visible tiles on the map
                    map.clear_visibility();
                    for pt in &vs.visible {
                        map[pt].revealed = true;
                        map[pt].visible = true;
                    }

                    // For renderable entities, hide those that are not in view
                    // and show those that are visible
                    for (e2, &Position(other), _, _) in
                        (&entities, &positions, !&players, &renders).join()
                    {
                        if vs.visible.contains(&other) {
                            hiddens.remove(e2);
                        } else {
                            hiddens.insert(e2, Hidden).unwrap();
                        }
                    }
                }
            }
        }
    }
}

/// System that manages entities that want to move in this turn.
#[derive(SystemDesc)]
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
            map[*from].blocked = false;
            map[to].blocked = true;
        }

        *from = to;
    }
}

impl<'s> System<'s> for MoveResolver {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        Entities<'s>,
        ReadStorage<'s, Player>,
        ReadStorage<'s, Faction>,
        ReadStorage<'s, CombatStats>,
        ReadStorage<'s, BlocksTile>,
        WriteStorage<'s, Position>,
        WriteStorage<'s, WantsToMove>,
        WriteStorage<'s, TargetedForMelee>,
        WriteStorage<'s, Viewshed>,
        Write<'s, Point>,
        Write<'s, WorldMap>,
    );

    fn run(
        &mut self,
        (
            entitites,
            players,
            factions,
            combatants,
            blockers,
            mut positions,
            mut movers,
            mut melee_targets,
            mut viewsheds,
            mut ppos,
            mut map,
        ): Self::SystemData,
    ) {
        for (e1, WantsToMove { to }) in (&entitites, movers.drain()).join() {
            if !map[to].blocked {
                if let Some(Position(p)) = positions.get_mut(e1) {
                    self.move_entity(e1, &mut map, p, to, blockers.contains(e1)); // update map state

                    // If the entity has a Viewshed, recompute it on movement
                    if let Some(vs) = viewsheds.get_mut(e1) {
                        vs.dirty = true;
                    }

                    // If the entity is the player, update its global position
                    if players.contains(e1) {
                        *ppos = to;
                    }
                }
            } else {
                let victims = (&entitites, &factions, &positions, &combatants);

                // If a fighter tries to moves tries to move into another fighter's tile
                // of a different faction, engage him in combat instead.
                if let Some(Faction(f1)) = factions.get(e1) {
                    for (victim, Faction(f2), Position(p2), _) in victims.join() {
                        if to == *p2 && f1 != f2 {
                            TargetedForMelee::target(&mut melee_targets, e1, victim);
                        }
                    }
                }
            }
        }
    }
}

/// Converts a `Position` component into a `Transform` component.
///
/// The default renderer works with pixels, while for this kind of game logic
/// it's easier to reason in terms of integer tile indexes.
/// This system takes this logical representation of position and turns it into
/// something that the rendering system can actually work with.
#[derive(SystemDesc)]
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
