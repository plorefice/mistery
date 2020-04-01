use crate::{
    components::{Monster, Viewshed},
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
        Read<'s, Point>,
    );

    fn run(&mut self, (viewsheds, monsters, player): Self::SystemData) {
        for (vs, _) in (&viewsheds, &monsters).join() {
            if vs.visible.contains(&player) {
                println!("The monster sees you.");
            }
        }
    }
}
