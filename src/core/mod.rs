//! This module contains the core of the game logic which does not fit into any ECS category.
//! This include the game initialization, map structure, entity spawning logic etc.

pub mod map;
pub mod spawn;

use map::WorldMap;

use crate::{
    components::*,
    renderer::WorldTileMap,
    resources::{CombatLog, TileDimension},
    ui::Ui,
};

use amethyst::{
    assets::{AssetStorage, Handle, Loader},
    core::{
        math::Vector3,
        transform::{Parent, Transform},
    },
    ecs::Entity,
    prelude::*,
    renderer::{Camera, ImageFormat, SpriteSheet, SpriteSheetFormat, Texture},
    window::ScreenDimensions,
};

/// This is the core game state. This is were the magic happens.
#[derive(Default)]
pub struct GameState {
    ui: Ui,
}

impl SimpleState for GameState {
    fn on_start(&mut self, StateData { world, .. }: StateData<'_, GameData>) {
        let sprite_sheet =
            load_sprite_sheet(world, "texture/cp437_20x20.png", "texture/cp437_20x20.ron");

        // Create required resources
        world.insert(TileDimension(20.0));
        world.insert({
            let mut log = CombatLog::default();
            log.push("Welcome to Mistery!");
            log
        });

        // Initialize world map (*must* come before everything else)
        create_map(world, 80, 50, sprite_sheet.clone());

        // Initialize all the game-related entities
        spawn_entities(world, sprite_sheet);

        // Load UI
        self.ui = Ui::new(world)
    }

    fn update(&mut self, StateData { world, .. }: &mut StateData<'_, GameData>) -> SimpleTrans {
        self.ui.refresh(world);
        Trans::None
    }
}

// Creates the world map.
fn create_map(world: &mut World, width: u32, height: u32, sheet: Handle<SpriteSheet>) {
    let tile_dim = world.read_resource::<TileDimension>().0 as u32;

    let tilemap = WorldTileMap::new(
        Vector3::new(width, height, 1),
        Vector3::new(tile_dim, tile_dim, 1),
        Some(sheet),
    );

    world.insert(WorldMap::rooms_and_corridors(width, height));

    world
        .create_entity()
        .with(Position([width / 2, height / 2 - 1].into()))
        .with(tilemap)
        .build();
}

// Spawns the player, the monsters and the camera.
fn spawn_entities(world: &mut World, sheet: Handle<SpriteSheet>) {
    // Iterator over all the map rooms
    let mut rooms = world
        .read_resource::<WorldMap>()
        .rooms()
        .to_vec()
        .into_iter();

    // Spawn the player in the middle of the first room.
    let player = spawn::player(world, rooms.next().unwrap().center(), sheet.clone());

    // Spawn random monsters in all the other rooms
    for room in rooms {
        spawn::random_monster(world, room.center(), sheet.clone());
    }

    // Finally, create the camera
    let (screen_width, screen_height) = {
        let dim = world.read_resource::<ScreenDimensions>();
        (dim.width(), dim.height())
    };
    spawn_camera(world, player, screen_width, screen_height);
}

// Creates an orthographic camera covering the entire screen view.
fn spawn_camera(world: &mut World, parent: Entity, screen_width: f32, screen_height: f32) {
    let mut position = Transform::default();
    position.set_translation_z(10.0);

    world
        .create_entity()
        .with(position)
        .with(Parent::new(parent))
        .with(Camera::standard_2d(screen_width, screen_height))
        .build();
}

// Loads an images and corresponding RON file as spritesheet.
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
