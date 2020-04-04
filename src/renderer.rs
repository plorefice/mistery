use crate::{
    map::{TileKind, WorldMap},
    math::Point,
    utils,
};

use amethyst::{
    core::math::Point3,
    ecs::World,
    prelude::*,
    renderer::palette::Srgba,
    tiles::{MortonEncoder, Tile, TileMap},
};

/// `TileMap` alias for `WorldTile` type.
pub type WorldTileMap = TileMap<WorldTile, MortonEncoder>;

struct WorldTileState {
    kind: TileKind,
    visible: bool,
}

#[derive(Default, Clone, Copy)]
pub struct WorldTile;

impl WorldTile {
    fn get(self, coordinates: Point3<u32>, world: &World) -> Option<WorldTileState> {
        let map = world.read_resource::<WorldMap>();

        // `Tile` coordinates grow right-down, while everything else in Amethyst
        // grows right-up, so the Y coordinate needs to be flipped before getting the tile.
        let p = Point::new(coordinates[0], map.height() - coordinates[1] - 1);

        map.revealed(p)
            .and_then(|revealed| match (revealed, map.get(p), map.visible(p)) {
                (true, Some(kind), Some(&visible)) => Some(WorldTileState { kind, visible }),
                _ => None,
            })
    }
}

impl Tile for WorldTile {
    fn sprite(&self, coordinates: Point3<u32>, world: &World) -> Option<usize> {
        self.get(coordinates, world)
            .map(|WorldTileState { kind, .. }| match kind {
                TileKind::Floor => utils::to_glyph('.'),
                TileKind::Wall => utils::to_glyph('#'),
            })
    }

    fn tint(&self, coordinates: Point3<u32>, world: &World) -> Srgba {
        self.get(coordinates, world)
            .map(|WorldTileState { kind, visible }| {
                if visible {
                    match kind {
                        TileKind::Floor => Srgba::new(0.2, 0.2, 0.2, 1.0),
                        TileKind::Wall => Srgba::new(0.0, 0.17, 0.21, 1.0),
                    }
                } else {
                    Srgba::new(0.05, 0.05, 0.05, 1.0)
                }
            })
            .unwrap()
    }
}
