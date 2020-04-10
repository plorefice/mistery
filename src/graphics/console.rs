use crate::{
    graphics::renderer::ConsoleTileMap,
    math::{Point, Rect},
    utils,
};

use amethyst::{
    core::math::Point3,
    renderer::palette::Srgba,
    tiles::{Map, MapStorage, Region},
};

pub trait Console {
    /// Clear the console.
    fn clear(&mut self);

    /// Prints a single line of text starting at the specified point.
    fn print<P, T>(&mut self, pt: P, text: T)
    where
        P: Into<Point>,
        T: AsRef<str>;

    /// Prints a single colored line of text starting at the specified point.
    fn print_color<P, T>(&mut self, pt: P, text: T, fg: Srgba)
    where
        P: Into<Point>,
        T: AsRef<str>;

    /// Draws a progress bar starting at the specified point.
    ///
    /// The progress bar will be `width` cells wide, with a value of `current` out of `max`.
    /// The filled portion will be colored with `fill`, and the empty portion with `empty`.
    fn draw_progress_bar<P: Into<Point>>(
        &mut self,
        pt: P,
        width: u32,
        current: u32,
        max: u32,
        fill: Srgba,
        empty: Srgba,
    );

    /// Draws a box along the rectangle-defined region using box-drawing characters.
    fn draw_box<R: Into<Rect>>(&mut self, rect: R) {
        let r = rect.into();
        let fg = Srgba::new(1., 1., 1., 1.);

        self.fill_region(r, ' ', Srgba::new(0., 0., 0., 1.));

        self.fill_region((r.left() + 1, r.top(), r.width() - 2, 1), '─', fg);
        self.fill_region((r.left() + 1, r.bottom(), r.width() - 2, 1), '─', fg);
        self.fill_region((r.left(), r.bottom() + 1, 1, r.height() - 2), '│', fg);
        self.fill_region((r.right(), r.bottom() + 1, 1, r.height() - 2), '│', fg);

        self.put((r.left(), r.bottom()), '┌', fg);
        self.put((r.right(), r.bottom()), '┐', fg);
        self.put((r.left(), r.top()), '└', fg);
        self.put((r.right(), r.top()), '┘', fg);
    }

    /// Fills a rectangle-defined region with a colored glyph.
    fn fill_region<R: Into<Rect>>(&mut self, rect: R, glyph: char, fg: Srgba);

    /// Puts a single colored glyph in the given cell.
    fn put<P: Into<Point>>(&mut self, pt: P, glyph: char, fg: Srgba);
}

impl Console for ConsoleTileMap {
    fn clear(&mut self) {
        let dims = *self.dimensions();

        for y in 0..dims[1] {
            for x in 0..dims[0] {
                if let Some(tile) = self.get_mut(&Point3::new(x, y, 0)) {
                    tile.glyph = None;
                }
            }
        }
    }

    fn print<P, T>(&mut self, pt: P, text: T)
    where
        P: Into<Point>,
        T: AsRef<str>,
    {
        self.print_color(pt, text, Srgba::new(1., 1., 1., 1.));
    }

    fn print_color<P, T>(&mut self, pt: P, text: T, fg: Srgba)
    where
        P: Into<Point>,
        T: AsRef<str>,
    {
        let text = text.as_ref();
        let pt = pt.into();

        let n = text.len() as u32;

        Region::new(
            Point3::new(pt.x(), pt.y(), 0),
            Point3::new(pt.x() + n - 1, pt.y(), 0),
        )
        .iter()
        .zip(text.chars())
        .for_each(|(pt, ch)| {
            if let Some(tile) = self.get_mut(&pt) {
                tile.glyph = Some(utils::to_glyph(ch));
                tile.tint = fg;
            }
        });
    }

    fn draw_progress_bar<P: Into<Point>>(
        &mut self,
        pt: P,
        width: u32,
        current: u32,
        max: u32,
        fill: Srgba,
        empty: Srgba,
    ) {
        let pt = pt.into();
        let ratio = current as f32 / max as f32;
        let filled = (ratio * width as f32).round() as u32;

        if filled > 0 {
            self.fill_region(Rect::new(pt.x(), pt.y(), filled, 1), '░', fill);
        }
        if filled < width {
            self.fill_region(
                Rect::new(pt.x() + filled, pt.y(), width - filled, 1),
                '░',
                empty,
            );
        }
    }

    fn fill_region<R: Into<Rect>>(&mut self, rect: R, glyph: char, fg: Srgba) {
        let rect = rect.into();

        for pt in &Region::new(
            Point3::new(rect.left(), rect.bottom(), 0),
            Point3::new(rect.right(), rect.top(), 0),
        ) {
            if let Some(tile) = self.get_mut(&pt) {
                tile.glyph = Some(utils::to_glyph(glyph));
                tile.tint = fg;
            }
        }
    }

    fn put<P: Into<Point>>(&mut self, pt: P, glyph: char, fg: Srgba) {
        let pt = pt.into();

        if let Some(tile) = self.get_mut(&Point3::new(pt.x(), pt.y(), 0)) {
            tile.glyph = Some(utils::to_glyph(glyph));
            tile.tint = fg;
        }
    }
}
