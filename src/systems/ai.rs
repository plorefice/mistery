use crate::{
    components::{Monster, Name, Position, Viewshed},
    map::{self, WorldMap},
    math::Point,
};

use amethyst::{
    derive::SystemDesc,
    ecs::{Join, Read, ReadStorage, System, SystemData, WriteStorage},
};

#[derive(SystemDesc)]
pub struct MonsterAI;

impl<'s> System<'s> for MonsterAI {
    type SystemData = (
        WriteStorage<'s, Position>,
        WriteStorage<'s, Viewshed>,
        ReadStorage<'s, Monster>,
        ReadStorage<'s, Name>,
        Read<'s, Point>,
        Read<'s, WorldMap>,
    );

    fn run(&mut self, (mut positions, mut viewsheds, monsters, _, player, map): Self::SystemData) {
        for (monster, vs, _) in (&mut positions, &mut viewsheds, &monsters).join() {
            let monster = &mut monster.0;
            if vs.visible.contains(&player) {
                if let Some(path) = map::a_star_search(&*map, monster, &*player) {
                    *monster = path[1]; // move monster
                    vs.dirty = true; // recompute viewshed
                }
            }
        }
    }
}
