use crate::{components::*, resources::CombatLog};

use amethyst::{
    core::Hidden,
    derive::SystemDesc,
    ecs::{Entities, Join, ReadStorage, System, SystemData, Write, WriteStorage},
};

/// System implementing the ability of entities to pick up other entities.
#[derive(SystemDesc)]
pub struct PickUpSystem;

impl<'s> System<'s> for PickUpSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        Entities<'s>,
        WriteStorage<'s, WantsToPickUp>,
        WriteStorage<'s, InBackpack>,
        WriteStorage<'s, Position>,
        WriteStorage<'s, Hidden>,
        ReadStorage<'s, Name>,
        Write<'s, CombatLog>,
    );

    fn run(
        &mut self,
        (entities, mut pickers, mut carried, mut positions, mut hiddens, names, mut log): Self::SystemData,
    ) {
        for (who, WantsToPickUp { what }) in (&entities, pickers.drain()).join() {
            positions.remove(what).unwrap();
            hiddens.insert(what, Hidden).unwrap(); // do not render entities being carried
            carried.insert(what, InBackpack { owner: who }).unwrap();

            if let (Some(Name(who)), Some(Name(what))) = (names.get(who), names.get(what)) {
                log.push(format!("{} picks up a {}.", who, what));
            }
        }
    }
}

/// System that resolve's an entity's intent to use an item.
#[derive(SystemDesc)]
pub struct ItemUsageResolver;

impl<'s> System<'s> for ItemUsageResolver {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        Entities<'s>,
        ReadStorage<'s, Name>,
        ReadStorage<'s, Consumable>,
        ReadStorage<'s, HealsUser>,
        WriteStorage<'s, WantsToUseItem>,
        WriteStorage<'s, CombatStats>,
        Write<'s, CombatLog>,
    );

    fn run(
        &mut self,
        (entities, names, consumables, healing, mut users, mut stats, mut log): Self::SystemData,
    ) {
        for (who, WantsToUseItem { what }) in (&entities, users.drain()).join() {
            // Healing item used by a unit with combat stats -> heal unit
            if let (Some(stats), Some(HealsUser { amount })) =
                (&mut stats.get_mut(who), healing.get(what))
            {
                stats.hp = i32::min(stats.max_hp, stats.hp + amount);

                log.push(format!(
                    "You use the {}, healing {} hp.",
                    names.get(what).map(|Name(n)| n.as_str()).unwrap_or("item"),
                    amount
                ))
            }

            if consumables.contains(what) {
                entities.delete(what).unwrap();
            }
        }
    }
}

/// System that resolve's an entity's intent to drop an item.
#[derive(SystemDesc)]
pub struct ItemDropResolver;

impl<'s> System<'s> for ItemDropResolver {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        ReadStorage<'s, Name>,
        WriteStorage<'s, WantsToDropItem>,
        WriteStorage<'s, Position>,
        WriteStorage<'s, InBackpack>,
        WriteStorage<'s, Hidden>,
        Write<'s, CombatLog>,
    );

    fn run(
        &mut self,
        (names, mut droppers, mut positions, mut carried, mut hiddens, mut log): Self::SystemData,
    ) {
        let locations = (&positions, droppers.drain())
            .join()
            .map(|(&Position(whereto), WantsToDropItem { what })| (what, whereto))
            .collect::<Vec<_>>();

        for (what, whereto) in locations {
            carried.remove(what).unwrap();
            hiddens.remove(what).unwrap();
            positions.insert(what, Position(whereto)).unwrap();

            log.push(format!(
                "You drop the {} on the floor.",
                names.get(what).map(|Name(n)| n.as_str()).unwrap_or("item"),
            ))
        }
    }
}
