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
    tiles::{MapStorage, MortonEncoder2D, Region, Tile, TileMap},
};

/// `TileMap` alias for `ConsoleTile` type.
pub type ConsoleTileMap = TileMap<ConsoleTile, MortonEncoder2D>;

/// Custom [`Tile`] implementation for the [`RenderTile2D`] plugin.
#[derive(Default, Clone, Copy)]
pub struct ConsoleTile {
    pub glyph: Option<usize>,
    pub tint: Srgba,
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
pub fn refresh_map_view(world: &mut World, tilemap: Entity) {
    let map = world.fetch::<WorldMap>();
    let width = map.width();
    let height = map.height();

    if let Some(tilemap) = world.write_storage::<ConsoleTileMap>().get_mut(tilemap) {
        for pt in &Region::new(Point3::new(0, 0, 0), Point3::new(width - 1, height - 1, 0)) {
            if let Some(tile) = tilemap.get_mut(&pt) {
                // `Tile` coordinates grow right-down, while everything else in Amethyst
                // grows right-up, so the Y coordinate needs to be flipped here.
                let state = map[Point::new(pt[0], height - pt[1] - 1)];

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
