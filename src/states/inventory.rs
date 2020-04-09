use crate::{
    components::*,
    graphics::{console::Console, renderer::ConsoleTileMap},
    states::{GameState, GameStateEvent, GameTrans},
    systems::ActionBinding,
};

use amethyst::{
    ecs::{Entity, Join},
    input::{is_close_requested, InputEvent},
    prelude::*,
    renderer::palette::Srgba,
};

pub enum Intent {
    UseItem,
    DropItem,
}

pub struct InventoryState {
    intent: Intent,
    console: Entity,
    item_list: Vec<(Entity, String)>,
}

impl InventoryState {
    pub fn new(intent: Intent, console: Entity) -> InventoryState {
        InventoryState {
            intent,
            console,
            item_list: Vec::new(),
        }
    }
}

impl GameState for InventoryState {
    fn on_start(&mut self, StateData { world, .. }: StateData<'_, GameData>) {
        // Store item list for later
        self.item_list = {
            let entities = world.entities();
            let players = world.read_storage::<Player>();
            let stored = world.read_storage::<InBackpack>();
            let named = world.read_storage::<Name>();

            (&entities, &stored, &named)
                .join()
                .filter_map(|(item, InBackpack { owner }, Name(name))| {
                    if players.contains(*owner) {
                        Some((item, name.clone()))
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
        };
    }

    fn update(&mut self, StateData { world, .. }: &mut StateData<'_, GameData>) -> GameTrans {
        let count = self.item_list.len() as u32;
        let (x, y, w, h) = (10, 10, 35, count + 4);

        let title_col = Srgba::new(1., 1., 0., 1.);
        let text_col = Srgba::new(1., 1., 1., 1.);

        if let Some(con) = world
            .write_storage::<ConsoleTileMap>()
            .get_mut(self.console)
        {
            con.draw_box((x, y, w, h));

            con.print_color((x + 2, y), " Inventory ", title_col);
            con.print_color((x + 2, y + h - 1), " Press ESC to cancel ", title_col);

            for (i, item) in self.item_list.iter().map(|(_, text)| text).enumerate() {
                let y = y + i as u32 + 2;
                con.put((x + 2, y), '(', text_col);
                con.put((x + 3, y), (b'a' + i as u8) as char, title_col);
                con.put((x + 4, y), ')', text_col);
                con.print((x + 6, y), item);
            }
        }

        Trans::None
    }

    fn on_stop(&mut self, StateData { world, .. }: StateData<'_, GameData>) {
        if let Some(con) = world
            .write_storage::<ConsoleTileMap>()
            .get_mut(self.console)
        {
            con.clear();
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
                        match self.intent {
                            Intent::UseItem => {
                                world
                                    .write_storage()
                                    .insert(player, WantsToUseItem { what: *what })
                                    .unwrap();
                            }
                            Intent::DropItem => {
                                world
                                    .write_storage()
                                    .insert(player, WantsToDropItem { what: *what })
                                    .unwrap();
                            }
                        }
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
