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
    #[allow(clippy::type_complexity)]
    type SystemData = (
        Entities<'s>,
        ReadStorage<'s, Player>,
        ReadStorage<'s, Faction>,
        ReadStorage<'s, Position>,
        ReadStorage<'s, Viewshed>,
        WriteStorage<'s, WantsToMove>,
        WriteStorage<'s, TargetedForMelee>,
        Read<'s, WorldMap>,
    );

    fn run(
        &mut self,
        (entities, players, factions, positions, viewsheds, mut movers, mut melee_targets, map): Self::SystemData,
    ) {
        let attackers = (&entities, !&players, &factions, &viewsheds, &positions);
        let targets = (&entities, &factions, &positions);

        for (attacker, _, Faction(f1), vs, &Position(p1)) in attackers.join() {
            for (target, Faction(f2), &Position(p2)) in targets.join() {
                // Skip not visibible and allies
                if f1 == f2 || !vs.visible.contains(&p2) {
                    break;
                }

                // If in range, target for combat, otherwise move closer.
                if math::distance_2d(p1, p2) == 1 {
                    TargetedForMelee::target(&mut melee_targets, attacker, target);
                } else if let Some(path) = map::a_star_search(&*map, p1, p2) {
                    movers
                        .insert(attacker, WantsToMove { to: path[1] })
                        .unwrap();
                }
            }
        }
    }
}
