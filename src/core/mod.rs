//! This module contains the core of the game logic which does not fit into any ECS category.
//! This include the game initialization, map structure, entity spawning logic etc.

pub mod map;

use map::WorldMap;

use crate::{
    components::*,
    renderer::WorldTileMap,
    resources::{CombatLog, TileDimension},
    ui::Ui,
    utils,
};

use amethyst::{
    assets::{AssetStorage, Handle, Loader},
    core::{
        math::Vector3,
        transform::{Parent, Transform},
        Hidden,
    },
    ecs::Entity,
    prelude::*,
    renderer::{
        palette::Srgba, resources::Tint, Camera, ImageFormat, SpriteRender, SpriteSheet,
        SpriteSheetFormat, Texture,
    },
    window::ScreenDimensions,
};

/// This is the core game state. This is were the magic happens.
#[derive(Default)]
pub struct GameState {
    ui: Ui,
}

impl SimpleState for GameState {
    fn on_start(&mut self, StateData { world, .. }: StateData<'_, GameData>) {
        let (screen_width, screen_height) = {
            let dim = world.read_resource::<ScreenDimensions>();
            (dim.width(), dim.height())
        };

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
        let player = spawn_player(world, sprite_sheet.clone());
        spawn_monsters(world, sprite_sheet);

        // Attach a camera to the player
        create_camera(world, player, screen_width, screen_height);

        // Load UI
        self.ui = Ui::new(world)
    }

    fn update(&mut self, StateData { world, .. }: &mut StateData<'_, GameData>) -> SimpleTrans {
        self.ui.refresh(world);
        Trans::None
    }
}

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

fn create_camera(world: &mut World, parent: Entity, screen_width: f32, screen_height: f32) {
    let mut position = Transform::default();
    position.set_translation_z(10.0);

    world
        .create_entity()
        .with(position)
        .with(Parent::new(parent))
        .with(Camera::standard_2d(screen_width, screen_height))
        .build();
}

fn spawn_player(world: &mut World, sheet: Handle<SpriteSheet>) -> Entity {
    let pos = world.read_resource::<WorldMap>().rooms()[0].center();

    // Insert player position as resource
    world.insert(pos);

    world
        .create_entity()
        .with(Player)
        .with(Faction(0))
        .with(InputListener)
        .with(ActsOnTurns::default())
        .with(Position(pos))
        .with(BlocksTile)
        .with(Viewshed::new(8))
        .with(CombatStats {
            max_hp: 30,
            hp: 30,
            defense: 2,
            power: 5,
        })
        .with(SpriteRender {
            sprite_sheet: sheet,
            sprite_number: utils::to_glyph('@'),
        })
        .with(Name("Hero".to_string()))
        .with(Tint(Srgba::new(0.7, 0.5, 0.0, 1.0)))
        .build()
}

fn spawn_monsters(world: &mut World, sheet: Handle<SpriteSheet>) {
    let spawn_points = world
        .read_resource::<WorldMap>()
        .rooms()
        .iter()
        .skip(1)
        .map(|r| r.center())
        .collect::<Vec<_>>();

    for (i, spawn_point) in spawn_points.into_iter().enumerate() {
        let (sprite, name) = if rand::random() {
            (utils::to_glyph('g'), "Goblin")
        } else {
            (utils::to_glyph('o'), "Orc")
        };

        world
            .create_entity()
            .with(Faction(1))
            .with(ActsOnTurns::default())
            .with(Position(spawn_point))
            .with(BlocksTile)
            .with(Viewshed::new(8))
            .with(CombatStats {
                max_hp: 16,
                hp: 16,
                defense: 1,
                power: 4,
            })
            .with(SpriteRender {
                sprite_sheet: sheet.clone(),
                sprite_number: sprite,
            })
            .with(Name(format!("{} #{}", name, i)))
            .with(Tint(Srgba::new(1.0, 0.0, 0.0, 1.0)))
            .with(Hidden) // initially monsters are not visible
            .build();
    }
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
