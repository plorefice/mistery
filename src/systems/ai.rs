use crate::components::Monster;

use amethyst::{
    derive::SystemDesc,
    ecs::{Join, ReadStorage, System, SystemData},
};

#[derive(SystemDesc)]
pub struct MonsterAI;

impl<'s> System<'s> for MonsterAI {
    type SystemData = ReadStorage<'s, Monster>;

    fn run(&mut self, monsters: Self::SystemData) {
        for _ in (&monsters).join() {
            println!("The monster is standing there thoughtless.");
        }
    }
}
