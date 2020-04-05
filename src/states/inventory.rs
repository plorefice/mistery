use crate::{
    components::*,
    states::{GameState, GameStateEvent, GameTrans},
    systems::ActionBinding,
};

use amethyst::{
    ecs::{Entity, Join},
    input::{is_close_requested, InputEvent},
    prelude::*,
    ui::{UiCreator, UiFinder, UiText, UiTransform},
};
use itertools::Itertools;

#[derive(Default)]
pub struct InventoryState {
    ui_handle: Option<Entity>,
    item_list: Vec<(Entity, String)>,
}

impl GameState for InventoryState {
    fn on_start(&mut self, StateData { world, .. }: StateData<'_, GameData>) {
        let handle =
            world.exec(|mut creator: UiCreator<'_>| creator.create("ui/inventory.ron", ()));

        // Store item list for later
        self.item_list = {
            let entities = world.entities();
            let players = world.read_storage::<Player>();
            let stored = world.read_storage::<InBackpack>();
            let named = world.read_storage::<Name>();

            (&entities, &stored, &named)
                .join()
                .enumerate()
                .filter_map(|(i, (item, InBackpack { owner }, Name(name)))| {
                    if players.contains(*owner) {
                        Some((
                            item,
                            format!("({}) {}", (b'a' + i as u8) as char, name.clone()),
                        ))
                    } else {
                        None
                    }
                })
                .collect_vec()
        };

        // Store handle to delete it later
        self.ui_handle = Some(handle);
    }

    fn update(&mut self, StateData { world, .. }: &mut StateData<'_, GameData>) -> GameTrans {
        let ui_container = world.exec(|f: UiFinder<'_>| f.find("inventory"));
        let ui_list = world.exec(|f: UiFinder<'_>| f.find("item-list"));

        if let (Some(container), Some(list)) = (ui_container, ui_list) {
            let mut font_size = 0.0;
            {
                let mut ui_text = world.write_storage::<UiText>();
                if let Some(list) = ui_text.get_mut(list) {
                    font_size = list.font_size;
                    list.text = self
                        .item_list
                        .iter()
                        .map(|(_, n)| n)
                        .cloned()
                        .intersperse("\n".to_string())
                        .collect();
                }
            }
            {
                let mut ui_transforms = world.write_storage::<UiTransform>();
                if let Some(container) = ui_transforms.get_mut(container) {
                    container.height = 40.0 + self.item_list.len() as f32 * font_size;
                }
            }
        }

        Trans::None
    }

    fn on_stop(&mut self, StateData { world, .. }: StateData<'_, GameData>) {
        // Destroy UI
        if let Some(handle) = self.ui_handle {
            world.delete_entity(handle).unwrap()
        }
    }

    fn handle_event(
        &mut self,
        StateData { world, .. }: StateData<'_, GameData>,
        event: GameStateEvent,
    ) -> GameTrans {
        match &event {
            StateEvent::Window(event) if is_close_requested(&event) => Trans::Quit,
            StateEvent::Input(InputEvent::ActionPressed(ActionBinding::Cancel)) => Trans::Pop,
            StateEvent::Input(InputEvent::KeyTyped(c @ 'a'..='z')) => {
                let i = ((*c as u8) - b'a') as usize;

                if let Some((what, _)) = self.item_list.get(i) {
                    if let Some((player, _)) = (&world.entities(), &world.read_storage::<Player>())
                        .join()
                        .next()
                    {
                        world
                            .write_storage()
                            .insert(player, WantsToUseItem { what: *what })
                            .unwrap();
                        Trans::Pop
                    } else {
                        Trans::None
                    }
                } else {
                    Trans::None
                }
            }
            _ => Trans::None,
        }
    }
}
