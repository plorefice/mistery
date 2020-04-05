use crate::{
    states::{GameState, GameStateEvent, GameTrans},
    systems::ActionBinding,
};

use amethyst::{
    input::{is_close_requested, InputEvent},
    prelude::*,
};

pub struct InventoryState;

impl GameState for InventoryState {
    fn handle_event(&mut self, _: StateData<'_, GameData>, event: GameStateEvent) -> GameTrans {
        match &event {
            StateEvent::Window(event) if is_close_requested(&event) => Trans::Quit,
            StateEvent::Input(InputEvent::ActionPressed(ActionBinding::Cancel)) => Trans::Pop,
            _ => Trans::None,
        }
    }
}
