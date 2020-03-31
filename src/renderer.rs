use crate::map::{TileKind, WorldMap};

use amethyst::{
    core::math::Point3,
    ecs::World,
    prelude::*,
    renderer::palette::Srgba,
    tiles::{MortonEncoder, Tile, TileMap},
};

/// `TileMap` alias for `WorldTile` type.
pub type WorldTileMap = TileMap<WorldTile, MortonEncoder>;

#[derive(Default, Clone, Copy)]
pub struct WorldTile;

impl WorldTile {
    fn get(&self, coordinates: Point3<u32>, world: &World) -> Option<TileKind> {
        let map = world.read_resource::<WorldMap>();

        // `Tile` coordinates grow right-down, while everything else in Amethyst
        // grows right-up, so the Y coordinate needs to be flipped before getting the tile.
        let (x, y) = (coordinates[0], map.height() - coordinates[1] - 1);
        map.is_revealed(x, y)
            .and_then(|viz| if viz { map.get(x, y) } else { None })
    }
}

impl Tile for WorldTile {
    fn sprite(&self, coordinates: Point3<u32>, world: &World) -> Option<usize> {
        self.get(coordinates, world).map(|k| match k {
            TileKind::Floor => 46,
            TileKind::Wall => 35,
        })
    }

    fn tint(&self, coordinates: Point3<u32>, world: &World) -> Srgba {
        self.get(coordinates, world)
            .map(|k| match k {
                TileKind::Floor => Srgba::new(0.2, 0.2, 0.2, 1.0),
                TileKind::Wall => Srgba::new(0.0, 0.17, 0.21, 1.0),
            })
            .unwrap()
    }
}
