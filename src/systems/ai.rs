use crate::{
    components::*,
    map::{self, WorldMap},
    math,
};

use amethyst::{
    derive::SystemDesc,
    ecs::{Entities, Join, Read, ReadStorage, System, SystemData, WriteStorage},
};

#[derive(SystemDesc)]
pub struct MonsterAI;

impl<'s> System<'s> for MonsterAI {
    type SystemData = (
        Entities<'s>,
        ReadStorage<'s, Player>,
        ReadStorage<'s, Faction>,
        ReadStorage<'s, Position>,
        ReadStorage<'s, Viewshed>,
        WriteStorage<'s, WantsToMove>,
        WriteStorage<'s, TargetedForCombat>,
        Read<'s, WorldMap>,
    );

    fn run(
        &mut self,
        (entities, players, factions, positions, viewsheds, mut movers, mut in_combat, map): Self::SystemData,
    ) {
        let monster_comps = (&entities, !&players, &factions, &viewsheds, &positions);
        let target_comps = (&entities, &factions, &positions);

        for (e1, _, Faction(f1), vs, &Position(p1)) in monster_comps.join() {
            for (e2, Faction(f2), &Position(p2)) in target_comps.join() {
                // Skip not visibible and allies
                if f1 == f2 || !vs.visible.contains(&p2) {
                    break;
                }

                // If in range, target for combat, otherwise move closer.
                if math::distance_2d(&p1, &p2) == 1 {
                    in_combat
                        .entry(e2)
                        .unwrap()
                        .or_insert(TargetedForCombat::default())
                        .by
                        .push(e1);
                } else if let Some(path) = map::a_star_search(&*map, &p1, &p2) {
                    movers.insert(e1, WantsToMove { to: path[1] }).unwrap();
                }
            }
        }
    }
}
