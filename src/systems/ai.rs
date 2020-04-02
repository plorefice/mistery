use crate::{
    components::{BlocksTile, Monster, Name, Position, Viewshed},
    map::{self, WorldMap},
    math::{self, Point},
};

use amethyst::{
    derive::SystemDesc,
    ecs::{Entities, Join, Read, ReadStorage, System, SystemData, Write, WriteStorage},
};

#[derive(SystemDesc)]
pub struct MonsterAI;

impl<'s> System<'s> for MonsterAI {
    type SystemData = (
        Entities<'s>,
        WriteStorage<'s, Position>,
        WriteStorage<'s, Viewshed>,
        ReadStorage<'s, Monster>,
        ReadStorage<'s, Name>,
        ReadStorage<'s, BlocksTile>,
        Read<'s, Point>,
        Write<'s, WorldMap>,
    );

    fn run(
        &mut self,
        (entities, mut positions, mut viewsheds, monsters, names, blockers, player, mut map): Self::SystemData,
    ) {
        for (e, monster, vs, name, _) in
            (&entities, &mut positions, &mut viewsheds, &names, &monsters).join()
        {
            let monster = &mut monster.0;
            if vs.visible.contains(&player) {
                if math::distance_2d(monster, &player) == 1 {
                    // We are in a tile adjacent to the player
                    println!("{} yells at you!", name.0);
                } else if let Some(path) = map::a_star_search(&*map, monster, &*player) {
                    let newpos = path[1];

                    // If the entity blocks tiles, change the blocked tile
                    if blockers.contains(e) {
                        map.blocked_mut(monster).and_then(|b| Some(*b = false));
                        map.blocked_mut(&newpos).and_then(|b| Some(*b = true));
                    }

                    *monster = newpos; // move monster
                    vs.dirty = true; // recompute viewshed
                }
            }
        }
    }
}
