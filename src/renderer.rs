use crate::{
    core::map::{TileKind, WorldMap},
    math::Point,
    utils,
};

use amethyst::{
    core::math::Point3,
    ecs::Entity,
    prelude::*,
    renderer::palette::Srgba,
    tiles::{Map, MapStorage, MortonEncoder, Region, Tile, TileMap},
};

/// `TileMap` alias for `ConsoleTile` type.
pub type ConsoleTileMap = TileMap<ConsoleTile, MortonEncoder>;

/// Custom [`Tile`] implementation for the [`RenderTile2D`] plugin.
#[derive(Clone, Copy)]
pub struct ConsoleTile {
    pub glyph: Option<usize>,
    pub tint: Srgba,
}

impl Default for ConsoleTile {
    fn default() -> Self {
        ConsoleTile {
            glyph: None,
            tint: Srgba::new(1., 1., 1., 1.),
        }
    }
}

impl Tile for ConsoleTile {
    fn sprite(&self, _: Point3<u32>, _: &World) -> Option<usize> {
        self.glyph
    }

    fn tint(&self, _: Point3<u32>, _: &World) -> Srgba {
        self.tint
    }
}

/// Updates the `ConsoleTileMap` to match the logical `WorldMap`.
pub fn refresh_map_view(world: &mut World, console: Entity) {
    let map = world.fetch::<WorldMap>();
    let player = world.fetch::<Point>();

    if let Some(console) = world.write_storage::<ConsoleTileMap>().get_mut(console) {
        let dims = *console.dimensions();

        // Offset of the bottom-left corner of the console in map coordinates
        let x_off = player.x() as i32 - (dims[0] as i32) / 2;
        let y_off = player.y() as i32 - (dims[1] as i32) / 2;

        for pt in &Region::new(Point3::new(0, 0, 0), Point3::new(dims[0], dims[1], 0)) {
            if let Some(tile) = console.get_mut(&pt) {
                // `Tile` coordinates grow right-down, while everything else in Amethyst
                // grows right-up, so the Y coordinate needs to be flipped here.
                let x = x_off + pt[0] as i32;
                let y = y_off + (dims[1] as i32) - pt[1] as i32 - 1;

                // Skip out of bound tiles
                if x < 0 || x >= map.width() as i32 || y < 0 || y >= map.height() as i32 {
                    tile.glyph = None;
                } else {
                    let state = map[Point::new(x as u32, y as u32)];

                    if state.revealed {
                        tile.glyph = Some(match state.kind {
                            TileKind::Floor => utils::to_glyph('.'),
                            TileKind::Wall => utils::to_glyph('#'),
                        });

                        tile.tint = if state.visible {
                            match state.kind {
                                TileKind::Floor => Srgba::new(0.2, 0.2, 0.2, 1.0),
                                TileKind::Wall => Srgba::new(0.0, 0.17, 0.21, 1.0),
                            }
                        } else {
                            Srgba::new(0.05, 0.05, 0.05, 1.0)
                        };
                    } else {
                        tile.glyph = None;
                    }
                }
            }
        }
    }
}
