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
