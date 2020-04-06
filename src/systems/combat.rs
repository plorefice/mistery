//! This module contains all the combat-related systems.

use crate::{components::*, resources::CombatLog};

use amethyst::{
    derive::SystemDesc,
    ecs::{Entities, Join, ReadStorage, System, SystemData, Write, WriteStorage},
};

/// Enum representing one of the possible turns in the state logic.
#[derive(Copy, Clone, PartialEq)]
pub enum Turn {
    Player,
    Others,
}

impl Default for Turn {
    fn default() -> Self {
        Turn::Player
    }
}

/// System that manages which entities get to act in the current turn.
///
/// At each invokation, the system checks which turns it's currently on,
/// and for each entity that should act on that turn, it checks if any of them
/// still has any AP left. If so, the turn keeps going until all entities
/// able to act have depleted their APs. Otherwise, the turn changes, and all
/// the entities that can act on the new turn have their AP replenished.
#[derive(Default, SystemDesc)]
pub struct TurnSystem {
    current: Turn,
}

impl<'s> System<'s> for TurnSystem {
    type SystemData = (WriteStorage<'s, ActsOnTurns>, ReadStorage<'s, Player>);

    fn run(&mut self, (mut actors, players): Self::SystemData) {
        match self.current {
            Turn::Player => {
                if (&actors, &players).join().any(|(a, _)| a.can_act()) {
                    return;
                }
                for (actor, _) in (&mut actors, !&players).join() {
                    actor.refresh();
                }
                self.current = Turn::Others;
            }
            Turn::Others => {
                if (&actors, !&players).join().any(|(a, _)| a.can_act()) {
                    return;
                }
                for (actor, _) in (&mut actors, &players).join() {
                    actor.refresh();
                }
                self.current = Turn::Player;
            }
        }
    }
}

/// Resolves melee combat between units.
///
/// For each defending unit, the system computes the actual damage that the entity will suffer
/// based on its defense and the attacker's power. Damage calculation is not performed right away,
/// rather the unit is simply tagged with the total amount of damage that it should take.
/// The [`DamageResolver`] handles the resolution of the damage itself.
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

/// Applies damage points to the units suffering damage.
///
/// The system iterates over all the units with a pending [`SufferDamage`] component
/// and subtracts the pending damage from their current HP. If a unit dies from the damage,
/// its entity is killed and later deleted.
#[derive(SystemDesc)]
pub struct DamageResolver;

impl<'s> System<'s> for DamageResolver {
    type SystemData = (
        Entities<'s>,
        ReadStorage<'s, Name>,
        WriteStorage<'s, SuffersDamage>,
        WriteStorage<'s, CombatStats>,
        Write<'s, CombatLog>,
    );

    fn run(&mut self, (entities, names, mut damages, mut combat_stats, mut log): Self::SystemData) {
        let damageds = (&entities, &names, damages.drain(), &mut combat_stats);

        for (e, Name(name), SuffersDamage { damage }, ref mut stats) in damageds.join() {
            stats.hp -= damage as i32;

            // If an entity drops below 0 HP, it dies
            if stats.hp <= 0 {
                log.push(format!("{} is dead.", name));
                entities.delete(e).unwrap();
            }
        }
    }
}
