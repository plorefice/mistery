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
        WriteStorage<'s, TargetedForCombat>,
    );

    fn run(&mut self, (names, combatants, mut in_combat): Self::SystemData) {
        for (Name(n1), _, TargetedForCombat { ref by }) in
            (&names, &combatants, in_combat.drain()).join()
        {
            for e2 in by {
                if let Some(Name(n2)) = names.get(*e2) {
                    println!("{} punches {}!", n2, n1);
                }
            }
        }
    }
}
