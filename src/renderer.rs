use crate::game::{TileKind, WorldMap};

use amethyst::{
    core::math::Point3,
    ecs::World,
    prelude::*,
    tiles::{MortonEncoder, Tile, TileMap},
};

/// `TileMap` alias for `WorldTile` type.
pub type WorldTileMap = TileMap<WorldTile, MortonEncoder>;

#[derive(Default, Clone, Copy)]
pub struct WorldTile;

impl Tile for WorldTile {
    fn sprite(&self, coordinates: Point3<u32>, world: &World) -> Option<usize> {
        let map = world.read_resource::<WorldMap>();
        let idx = coordinates[1] * map.width + coordinates[0];

        match map.tiles[idx as usize] {
            TileKind::Floor => Some(46),
            TileKind::Wall => Some(35),
        }
    }
}
