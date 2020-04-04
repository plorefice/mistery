use crate::{components::*, game::CombatLog};

use amethyst::{
    derive::SystemDesc,
    ecs::{Entities, Join, ReadStorage, System, SystemData, Write, WriteStorage},
};

#[derive(SystemDesc)]
pub struct MeleeCombatResolver;

impl<'s> System<'s> for MeleeCombatResolver {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        Entities<'s>,
        ReadStorage<'s, Name>,
        ReadStorage<'s, CombatStats>,
        WriteStorage<'s, TargetedForMelee>,
        WriteStorage<'s, SuffersDamage>,
        Write<'s, CombatLog>,
    );

    fn run(
        &mut self,
        (entities, names, combat_stats, mut melee_targets, mut damage, mut log): Self::SystemData,
    ) {
        let defenders = (&entities, &names, &combat_stats, melee_targets.drain());

        for (defender, Name(def_name), def_stats, TargetedForMelee { by: ref attackers }) in
            defenders.join()
        {
            for attacker in attackers {
                let Name(atk_name) = names.get(*attacker).unwrap();
                let atk_stats = combat_stats.get(*attacker).unwrap();

                let dmg = i32::max(0, atk_stats.power - def_stats.defense);

                if dmg > 0 {
                    log.push(format!("{} hits {} for {} hp.", atk_name, def_name, dmg));
                    SuffersDamage::damage(&mut damage, defender, dmg as u32);
                } else {
                    log.push(format!("{} cannot hit {}.", atk_name, def_name));
                }
            }
        }
    }
}

#[derive(Default, SystemDesc)]
pub struct DamageResolver;

impl<'s> System<'s> for DamageResolver {
    type SystemData = (
        Entities<'s>,
        WriteStorage<'s, SuffersDamage>,
        WriteStorage<'s, CombatStats>,
    );

    fn run(&mut self, (entities, mut damages, mut combat_stats): Self::SystemData) {
        let damageds = (&entities, damages.drain(), &mut combat_stats);

        for (e, SuffersDamage { damage }, stats) in damageds.join() {
            stats.hp -= damage as i32;

            // If an entity drops below 0 HP, it dies
            if stats.hp <= 0 {
                entities.delete(e).unwrap();
            }
        }
    }
}
