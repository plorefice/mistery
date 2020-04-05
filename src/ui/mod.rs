use crate::{components::*, resources::CombatLog};

use amethyst::{
    ecs::{Entity, Join},
    prelude::*,
    ui::{UiCreator, UiFinder, UiText, UiTransform},
    utils::fps_counter::FpsCounter,
};
use itertools::Itertools;
use std::collections::HashMap;

/// Wrapper around top-level UI functions.
#[derive(Default)]
pub struct Ui {
    ui_cache: HashMap<String, Entity>,
}

impl Ui {
    /// Initializes the user interface.
    pub fn new(world: &mut World) -> Ui {
        // Load UI prefabs
        world.exec(|mut creator: UiCreator<'_>| {
            creator.create("ui/fps-counter.ron", ());
            creator.create("ui/infobox.ron", ());
        });

        Ui::default()
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
        if let Some(fps) = self.get_ui_element("fps-counter", world) {
            let mut ui_text = world.write_storage::<UiText>();
            if let Some(counter) = ui_text.get_mut(fps) {
                counter.text = format!(
                    "{:.0}",
                    world.read_resource::<FpsCounter>().sampled_fps().round()
                );
            }
        }
    }

    // Update the HP text and bar in the infobox.
    fn update_hp_display(&mut self, world: &mut World) {
        let hp_text = self.get_ui_element("hp-text", world);
        let hp_slot = self.get_ui_element("hp-slot", world);
        let hp_bar = self.get_ui_element("hp-bar", world);

        if let (Some(hp_text), Some(hp_slot), Some(hp_bar)) = (hp_text, hp_slot, hp_bar) {
            let players = world.read_storage::<Player>();
            let stats = world.read_storage::<CombatStats>();

            let mut ui_text = world.write_storage::<UiText>();
            let mut ui_transform = world.write_storage::<UiTransform>();

            if let Some((_, stats)) = (&players, &stats).join().next() {
                if let Some(hp) = ui_text.get_mut(hp_text) {
                    hp.text = format!("HP: {} / {}", stats.hp, stats.max_hp);
                }

                let ratio = stats.hp as f32 / stats.max_hp as f32;
                let width = if let Some(hp_slot) = ui_transform.get_mut(hp_slot) {
                    hp_slot.width
                } else {
                    return;
                };

                if let Some(hp_bar) = ui_transform.get_mut(hp_bar) {
                    hp_bar.width = width * ratio;
                }
            }
        }
    }

    // Update the combat log to show the most recent messages.
    fn update_combat_log(&mut self, world: &mut World) {
        if let Some(ui_log) = self.get_ui_element("combat-log", world) {
            let mut ui_text = world.write_storage::<UiText>();
            if let Some(ui_log) = ui_text.get_mut(ui_log) {
                ui_log.text = world
                    .read_resource::<CombatLog>()
                    .lines()
                    .iter()
                    .rev()
                    .take(3)
                    .rev()
                    .cloned()
                    .intersperse("\n".to_string())
                    .collect::<String>();
            }
        }
    }

    /// Returns the entity corresponding to the UI element with the given ID, if it exists.
    fn get_ui_element<S: AsRef<str>>(&mut self, id: S, world: &mut World) -> Option<Entity> {
        match self.ui_cache.get(id.as_ref()) {
            Some(e) => Some(*e),
            None => {
                if let Some(e) = world.exec(|f: UiFinder<'_>| f.find(id.as_ref())) {
                    self.ui_cache.insert(id.as_ref().to_owned(), e)
                } else {
                    None
                }
            }
        }
    }
}
