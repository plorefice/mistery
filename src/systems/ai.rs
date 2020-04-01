use crate::{
    components::{Monster, Name, Viewshed},
    math::Point,
};

use amethyst::{
    derive::SystemDesc,
    ecs::{Join, Read, ReadStorage, System, SystemData},
};

#[derive(SystemDesc)]
pub struct MonsterAI;

impl<'s> System<'s> for MonsterAI {
    type SystemData = (
        ReadStorage<'s, Viewshed>,
        ReadStorage<'s, Monster>,
        ReadStorage<'s, Name>,
        Read<'s, Point>,
    );

    fn run(&mut self, (viewsheds, monsters, names, player): Self::SystemData) {
        for (vs, _, name) in (&viewsheds, &monsters, &names).join() {
            if vs.visible.contains(&player) {
                println!("{} shouts at you!", name.0);
            }
        }
    }
}
