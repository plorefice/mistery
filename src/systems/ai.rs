//! This module contains all the AI-related systems.

use crate::{
    components::*,
    core::map::{self, WorldMap},
    math,
};

use amethyst::{
    derive::SystemDesc,
    ecs::{Entities, Join, Read, ReadStorage, System, SystemData, WriteStorage},
};

/// Monster logic processing.
///
/// For each monster in the field, the system checks if any player unit is in its FoV
/// and either chases it, or if it is in an adjacent tiles, tries to attack.
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
        WriteStorage<'s, ActsOnTurns>,
        WriteStorage<'s, WantsToMove>,
        WriteStorage<'s, TargetedForMelee>,
        Read<'s, WorldMap>,
    );

    fn run(
        &mut self,
        (
            entities,
            players,
            factions,
            positions,
            viewsheds,
            mut actors,
            mut movers,
            mut melee_targets,
            map,
        ): Self::SystemData,
    ) {
        let attackers = (
            &entities,
            &mut actors,
            &factions,
            &viewsheds,
            &positions,
            !&players,
        );

        let targets = (&entities, &factions, &positions);

        for (attacker, actor, &Faction(f1), vs, &Position(p1), _) in attackers.join() {
            if !actor.perform() {
                continue;
            }

            for (target, &Faction(f2), &Position(p2)) in targets.join() {
                // Skip not visibible and allies
                if f1 == f2 || !vs.visible.contains(&p2) {
                    continue;
                }

                // If in range, target for combat, otherwise move closer.
                if math::distance_2d(p1, p2) == 1 {
                    TargetedForMelee::target(&mut melee_targets, attacker, target);
                } else if let Some(path) = map::a_star_search(&*map, p1, p2) {
                    movers
                        .insert(attacker, WantsToMove { to: path[1] })
                        .unwrap();
                }

                // Don't chase multiple units!
                break;
            }
        }
    }
}
