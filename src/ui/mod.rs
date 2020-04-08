use crate::{renderer::ConsoleTileMap, utils};

use amethyst::{
    core::math::Point3,
    ecs::Entity,
    prelude::*,
    tiles::{MapStorage, Region},
    utils::fps_counter::FpsCounter,
};

/// Wrapper around top-level UI functions.
pub struct Ui {
    console: Entity,
}

impl Ui {
    /// Initializes the user interface.
    pub fn new(console: Entity) -> Ui {
        Ui { console }
    }

    /// Refreshes all the UI components.
    pub fn refresh(&mut self, world: &mut World) {
        self.update_infobox(world);
        self.update_fps_counter(world);
    }

    // Updates the infobox to reflect the current game state.
    fn update_infobox(&mut self, world: &mut World) {
        self.update_hp_display(world);
        self.update_combat_log(world);
    }

    // Updates the FPS counter with the currently measured FPS.
    fn update_fps_counter(&mut self, world: &mut World) {
        if let Some(console) = world.write_storage().get_mut(self.console) {
            self.draw_text(
                console,
                0,
                0,
                format!(
                    "{:.0}",
                    world.read_resource::<FpsCounter>().sampled_fps().round()
                ),
            );
        }
    }

    // Update the HP text and bar in the infobox.
    fn update_hp_display(&mut self, _world: &mut World) {
        // let hp_text = self.get_ui_element("hp-text", world);
        // let hp_slot = self.get_ui_element("hp-slot", world);
        // let hp_bar = self.get_ui_element("hp-bar", world);

        // if let (Some(hp_text), Some(hp_slot), Some(hp_bar)) = (hp_text, hp_slot, hp_bar) {
        //     let players = world.read_storage::<Player>();
        //     let stats = world.read_storage::<CombatStats>();

        //     let mut ui_text = world.write_storage::<UiText>();
        //     let mut ui_transform = world.write_storage::<UiTransform>();

        //     if let Some((_, stats)) = (&players, &stats).join().next() {
        //         if let Some(hp) = ui_text.get_mut(hp_text) {
        //             hp.text = format!("HP: {} / {}", stats.hp, stats.max_hp);
        //         }

        //         let ratio = stats.hp as f32 / stats.max_hp as f32;
        //         let width = if let Some(hp_slot) = ui_transform.get_mut(hp_slot) {
        //             hp_slot.width
        //         } else {
        //             return;
        //         };

        //         if let Some(hp_bar) = ui_transform.get_mut(hp_bar) {
        //             hp_bar.width = width * ratio;
        //         }
        //     }
        // }
    }

    // Update the combat log to show the most recent messages.
    fn update_combat_log(&mut self, _world: &mut World) {
        // if let Some(ui_log) = self.get_ui_element("combat-log", world) {
        //     let mut ui_text = world.write_storage::<UiText>();
        //     if let Some(ui_log) = ui_text.get_mut(ui_log) {
        //         ui_log.text = world
        //             .read_resource::<CombatLog>()
        //             .lines()
        //             .iter()
        //             .rev()
        //             .take(3)
        //             .rev()
        //             .cloned()
        //             .intersperse("\n".to_string())
        //             .collect::<String>();
        //     }
        // }
    }

    fn draw_text<T: AsRef<str>>(&mut self, console: &mut ConsoleTileMap, x: u32, y: u32, text: T) {
        let text = text.as_ref();
        let n = text.len() as u32;

        Region::new(Point3::new(x, y, 1), Point3::new(x + n - 1, y, 1))
            .iter()
            .zip(text.chars())
            .for_each(|(pt, ch)| {
                if let Some(tile) = console.get_mut(&pt) {
                    tile.glyph = Some(utils::to_glyph(ch));
                }
            });
    }
}
