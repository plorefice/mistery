use crate::{
    components::{Monster, Name, Position, Viewshed, WantsToMove},
    map::{self, WorldMap},
    math::{self, Point},
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
        ReadStorage<'s, Monster>,
        ReadStorage<'s, Position>,
        ReadStorage<'s, Viewshed>,
        ReadStorage<'s, Name>,
        WriteStorage<'s, WantsToMove>,
        Read<'s, Point>,
        Read<'s, WorldMap>,
    );

    fn run(
        &mut self,
        (entities, monsters, positions, viewsheds, names, mut movers, player, map): Self::SystemData,
    ) {
        for (e, _, vs, &Position(p), name) in
            (&entities, &monsters, &viewsheds, &positions, &names).join()
        {
            if vs.visible.contains(&player) {
                // If we are in a tile adjacent to the player, yell at him, otherwise move
                if math::distance_2d(&p, &player) == 1 {
                    println!("{} yells at you!", name.0);
                } else if let Some(path) = map::a_star_search(&*map, &p, &*player) {
                    movers.insert(e, WantsToMove { to: path[1] }).unwrap();
                }
            }
        }
    }
}
