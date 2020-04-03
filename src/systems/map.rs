use crate::{
    components::*,
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
        for (e1, &Position(pos), vs) in (&entities, &positions, &mut viewsheds).join() {
            if vs.dirty {
                vs.visible = ShadowcastFoV::run(&*map, pos[0], pos[1], vs.range);
                vs.dirty = false;

                // If the entity is also a player, perform some additional actions
                if players.contains(e1) {
                    // First, reveal the visible tiles on the map
                    map.clear_visibility();
                    for pt in &vs.visible {
                        map.revealed_mut(pt).and_then(|rev| Some(*rev = true));
                        map.visible_mut(pt).and_then(|viz| Some(*viz = true));
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
        ReadStorage<'s, Faction>,
        ReadStorage<'s, CombatStats>,
        ReadStorage<'s, BlocksTile>,
        WriteStorage<'s, Position>,
        WriteStorage<'s, WantsToMove>,
        WriteStorage<'s, TargetedForCombat>,
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
            mut in_combat,
            mut viewsheds,
            mut ppos,
            mut map,
        ): Self::SystemData,
    ) {
        for (e1, WantsToMove { to }) in (&entitites, movers.drain()).join() {
            match map.blocked(&to) {
                Some(&false) => {
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
                }
                Some(&true) => {
                    // If a fighter tries to moves tries to move into another fighter's tile
                    // of a different faction, engage him in combat instead.
                    if let Some(Faction(f1)) = factions.get(e1) {
                        for (e2, Faction(f2), Position(p2), _) in
                            (&entitites, &factions, &positions, &combatants).join()
                        {
                            if to == *p2 && f1 != f2 {
                                in_combat
                                    .entry(e2)
                                    .unwrap()
                                    .or_insert(TargetedForCombat::default())
                                    .by
                                    .push(e1);
                            }
                        }
                    }
                }
                _ => (),
            }
        }
    }
}
