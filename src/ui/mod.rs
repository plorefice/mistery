use crate::{
    components::{CombatStats, Player},
    math::{Point, Rect},
    renderer::ConsoleTileMap,
    resources::CombatLog,
    utils,
};

use amethyst::{
    core::math::Point3,
    ecs::{Entity, Join},
    prelude::*,
    renderer::palette::Srgba,
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
        if let Some(console) = world.write_storage().get_mut(self.console) {
            self.draw_box(console, Rect::new(0, 43, 80, 7));
        }
        self.update_combat_log(world);
        self.update_hp_display(world);
    }

    // Updates the FPS counter with the currently measured FPS.
    fn update_fps_counter(&mut self, world: &mut World) {
        if let Some(console) = world.write_storage().get_mut(self.console) {
            let fps = format!(
                "{:.0}",
                world.read_resource::<FpsCounter>().sampled_fps().round()
            );
            self.print(console, (0, 0), fps);
        }
    }

    // Update the HP text and bar in the infobox.
    fn update_hp_display(&mut self, world: &mut World) {
        if let Some(console) = world.write_storage().get_mut(self.console) {
            let players = world.read_storage::<Player>();
            let stats = world.read_storage::<CombatStats>();

            if let Some((_, stats)) = (&players, &stats).join().next() {
                self.print_color(
                    console,
                    (12, 43),
                    format!(" HP: {} / {} ", stats.hp, stats.max_hp),
                    Srgba::new(1., 1., 0., 1.),
                );

                self.draw_progress(
                    console,
                    (28, 43),
                    51,
                    stats.hp as u32,
                    stats.max_hp as u32,
                    Srgba::new(1., 0., 0., 1.),
                );
            }
        }
    }

    // Update the combat log to show the most recent messages.
    fn update_combat_log(&mut self, world: &mut World) {
        if let Some(console) = world.write_storage().get_mut(self.console) {
            for (i, line) in world
                .read_resource::<CombatLog>()
                .lines()
                .iter()
                .rev()
                .take(5)
                .rev()
                .enumerate()
            {
                self.print(console, (1, 44 + i as u32), line);
            }
        }
    }

    fn print<P, T>(&mut self, console: &mut ConsoleTileMap, pt: P, text: T)
    where
        P: Into<Point>,
        T: AsRef<str>,
    {
        self.print_color(console, pt, text, Srgba::new(1., 1., 1., 1.));
    }

    fn print_color<P, T>(&mut self, console: &mut ConsoleTileMap, pt: P, text: T, fg: Srgba)
    where
        P: Into<Point>,
        T: AsRef<str>,
    {
        let text = text.as_ref();
        let pt = pt.into();

        let n = text.len() as u32;

        Region::new(
            Point3::new(pt.x(), pt.y(), 1),
            Point3::new(pt.x() + n - 1, pt.y(), 1),
        )
        .iter()
        .zip(text.chars())
        .for_each(|(pt, ch)| {
            if let Some(tile) = console.get_mut(&pt) {
                tile.glyph = Some(utils::to_glyph(ch));
                tile.tint = fg;
            }
        });
    }

    fn draw_box(&mut self, console: &mut ConsoleTileMap, r: Rect) {
        let fg = Srgba::new(1., 1., 1., 1.);

        self.fill_region(console, ' ', r, Srgba::new(0., 0., 0., 1.));

        self.put(console, '┌', (r.left(), r.bottom()), fg);
        self.put(console, '┐', (r.right(), r.bottom()), fg);
        self.put(console, '└', (r.left(), r.top()), fg);
        self.put(console, '┘', (r.right(), r.top()), fg);

        self.draw_line(console, r.left() + 1, r.right() - 1, r.top(), fg);
        self.draw_line(console, r.left() + 1, r.right() - 1, r.bottom(), fg);
        self.draw_vline(console, r.bottom() + 1, r.top() - 1, r.left(), fg);
        self.draw_vline(console, r.bottom() + 1, r.top() - 1, r.right(), fg);
    }

    fn draw_line(&mut self, console: &mut ConsoleTileMap, x1: u32, x2: u32, y: u32, fg: Srgba) {
        self.fill_region(console, '─', Rect::new(x1, y, x2 - x1 + 1, 1), fg);
    }

    fn draw_vline(&mut self, console: &mut ConsoleTileMap, y1: u32, y2: u32, x: u32, fg: Srgba) {
        self.fill_region(console, '│', Rect::new(x, y1, 1, y2 - y1 + 1), fg);
    }

    fn draw_progress<P: Into<Point>>(
        &mut self,
        console: &mut ConsoleTileMap,
        pt: P,
        width: u32,
        value: u32,
        max: u32,
        fg: Srgba,
    ) {
        let pt = pt.into();
        let ratio = value as f32 / max as f32;
        let fill = (ratio * width as f32).round() as u32;

        if fill > 0 {
            self.fill_region(console, '░', Rect::new(pt.x(), pt.y(), fill, 1), fg);
        }
        if fill < width {
            self.fill_region(
                console,
                '░',
                Rect::new(pt.x() + fill, pt.y(), width - fill, 1),
                Srgba::new(fg.red * 0.2, fg.green * 0.2, fg.blue * 0.2, 1.0),
            );
        }
    }

    fn fill_region(&mut self, console: &mut ConsoleTileMap, glyph: char, rect: Rect, fg: Srgba) {
        for pt in &Region::new(
            Point3::new(rect.left(), rect.bottom(), 1),
            Point3::new(rect.right(), rect.top(), 1),
        ) {
            if let Some(tile) = console.get_mut(&pt) {
                tile.glyph = Some(utils::to_glyph(glyph));
                tile.tint = fg;
            }
        }
    }

    fn put<P: Into<Point>>(&mut self, console: &mut ConsoleTileMap, glyph: char, pt: P, fg: Srgba) {
        let pt = pt.into();

        if let Some(tile) = console.get_mut(&Point3::new(pt.x(), pt.y(), 1)) {
            tile.glyph = Some(utils::to_glyph(glyph));
            tile.tint = fg;
        }
    }
}
