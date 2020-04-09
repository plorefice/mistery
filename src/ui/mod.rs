mod console;

pub use console::Console;

use crate::{
    components::{CombatStats, Player},
    renderer::ConsoleTileMap,
    resources::CombatLog,
};

use amethyst::{
    ecs::{Entity, Join},
    prelude::*,
    renderer::palette::Srgba,
    utils::fps_counter::FpsCounter,
};

// Less typing :)
type CTM = ConsoleTileMap;

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
        if let Some(con) = world.write_storage::<CTM>().get_mut(self.console) {
            con.draw_box((0, 43, 80, 7));
        }
        self.update_combat_log(world);
        self.update_hp_display(world);
    }

    // Updates the FPS counter with the currently measured FPS.
    fn update_fps_counter(&mut self, world: &mut World) {
        let fps = format!(
            "{:.0}",
            world.read_resource::<FpsCounter>().sampled_fps().round()
        );

        if let Some(con) = world.write_storage::<CTM>().get_mut(self.console) {
            con.print((0, 0), fps);
        }
    }

    // Update the HP text and bar in the infobox.
    fn update_hp_display(&mut self, world: &mut World) {
        if let Some(con) = world.write_storage::<CTM>().get_mut(self.console) {
            let players = world.read_storage::<Player>();
            let stats = world.read_storage::<CombatStats>();

            if let Some((_, stats)) = (&players, &stats).join().next() {
                con.print_color(
                    (12, 43),
                    format!(" HP: {} / {} ", stats.hp, stats.max_hp),
                    Srgba::new(1., 1., 0., 1.),
                );

                con.draw_progress_bar(
                    (28, 43),
                    51,
                    stats.hp as u32,
                    stats.max_hp as u32,
                    Srgba::new(1., 0., 0., 1.),
                    Srgba::new(0.2, 0., 0., 1.),
                );
            }
        }
    }

    // Update the combat log to show the most recent messages.
    fn update_combat_log(&mut self, world: &mut World) {
        if let Some(con) = world.write_storage::<CTM>().get_mut(self.console) {
            for (i, line) in world
                .read_resource::<CombatLog>()
                .lines()
                .iter()
                .rev()
                .take(5)
                .rev()
                .enumerate()
            {
                con.print((1, 44 + i as u32), line);
            }
        }
    }
}
