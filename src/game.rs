use crate::{components::*, renderer::WorldTileMap};

use amethyst::{
    assets::{AssetStorage, Handle, Loader},
    core::{math::Vector3, transform::Transform},
    prelude::*,
    renderer::{
        palette::Srgba, resources::Tint, Camera, ImageFormat, SpriteRender, SpriteSheet,
        SpriteSheetFormat, Texture,
    },
    window::ScreenDimensions,
};

/// Resource holding the side length of a tile.
#[derive(Default)]
pub struct TileDimension(pub f32);

pub struct GameState;

impl SimpleState for GameState {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let StateData { world, .. } = data;

        let (screen_width, screen_height) = {
            let dim = world.read_resource::<ScreenDimensions>();
            (dim.width(), dim.height())
        };

        let sprite_sheet =
            load_sprite_sheet(world, "texture/cp437_20x20.png", "texture/cp437_20x20.ron");

        // FIXME: compute this parameter somehow.
        world.insert(TileDimension(20.0));

        // Create player entity
        world
            .create_entity()
            .with(InputListener)
            .with(Position([40, 25].into()))
            .with(SpriteRender {
                sprite_sheet: sprite_sheet.clone(),
                sprite_number: 64,
            })
            .with(Tint(Srgba::new(1.0, 1.0, 0.0, 1.0)))
            .build();

        // Create world map
        initialize_map(world, 80, 50, sprite_sheet);

        // Create camera
        initialize_camera(world, screen_width, screen_height);
    }
}

fn initialize_map(world: &mut World, width: u32, height: u32, sheet: Handle<SpriteSheet>) {
    let tile_dim = world.read_resource::<TileDimension>().0 as u32;

    let tilemap = WorldTileMap::new(
        Vector3::new(width, height, 1),
        Vector3::new(tile_dim, tile_dim, 1),
        Some(sheet),
    );

    world.insert(WorldMap::random(width, height));

    world
        .create_entity()
        .with(Position([width as i32 / 2, height as i32 / 2 - 1].into()))
        .with(tilemap)
        .build();
}

fn initialize_camera(world: &mut World, screen_width: f32, screen_height: f32) {
    let tile_dim = world.read_resource::<TileDimension>().0;

    let mut position = Transform::default();
    position.set_translation_xyz(
        (screen_width - tile_dim) / 2.0,
        (screen_height - tile_dim) / 2.0,
        1.0,
    );

    world
        .create_entity()
        .with(position)
        .with(Camera::standard_2d(screen_width, screen_height))
        .build();
}

fn load_sprite_sheet(world: &mut World, png_path: &str, ron_path: &str) -> Handle<SpriteSheet> {
    let texture_handle = {
        let loader = world.read_resource::<Loader>();
        let texture_storage = world.read_resource::<AssetStorage<Texture>>();
        loader.load(png_path, ImageFormat::default(), (), &texture_storage)
    };

    let loader = world.read_resource::<Loader>();
    let sprite_sheet_store = world.read_resource::<AssetStorage<SpriteSheet>>();

    loader.load(
        ron_path,
        SpriteSheetFormat(texture_handle),
        (),
        &sprite_sheet_store,
    )
}

#[derive(Clone, Copy)]
pub enum TileKind {
    Wall,
    Floor,
}

pub struct WorldMap {
    pub width: u32,
    pub tiles: Vec<TileKind>,
}

impl WorldMap {
    fn random(width: u32, height: u32) -> WorldMap {
        let mut tiles = Vec::with_capacity((width * height) as usize);

        for y in 0..height {
            for x in 0..width {
                tiles.push(if y == 0 || y == height - 1 || x == 0 || x == width - 1 {
                    TileKind::Wall
                } else if rand::random::<f32>() < 0.1 {
                    TileKind::Wall
                } else {
                    TileKind::Floor
                });
            }
        }

        WorldMap { width, tiles }
    }
}
