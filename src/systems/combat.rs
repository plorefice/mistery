use crate::components::*;

use amethyst::{
    derive::SystemDesc,
    ecs::{Join, ReadStorage, System, SystemData, WriteStorage},
};

#[derive(SystemDesc)]
pub struct CombatResolver;

impl<'s> System<'s> for CombatResolver {
    type SystemData = (
        ReadStorage<'s, Name>,
        ReadStorage<'s, CombatStats>,
        WriteStorage<'s, TargetedForMelee>,
    );

    fn run(&mut self, (names, combatants, mut melee_targets): Self::SystemData) {
        for (Name(n1), _, TargetedForMelee { ref by }) in
            (&names, &combatants, melee_targets.drain()).join()
        {
            for e2 in by {
                if let Some(Name(n2)) = names.get(*e2) {
                    println!("{} punches {}!", n2, n1);
                }
            }
        }
    }
}
