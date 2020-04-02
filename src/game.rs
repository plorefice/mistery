use crate::{
    components::*,
    map::WorldMap,
    renderer::WorldTileMap,
    systems::{
        ai::MonsterAI,
        map::{MapIndexingSystem, VisibilitySystem},
        InputDispatcher, PositionTranslator,
    },
    utils,
};

use amethyst::{
    assets::{AssetStorage, Handle, Loader},
    core::{math::Vector3, transform::Transform, ArcThreadPool, Hidden, Time},
    ecs::{Dispatcher, DispatcherBuilder, Entity},
    prelude::*,
    renderer::{
        palette::Srgba, resources::Tint, Camera, ImageFormat, SpriteRender, SpriteSheet,
        SpriteSheetFormat, Texture,
    },
    ui::{Anchor, TtfFormat, UiText, UiTransform},
    utils::fps_counter::FpsCounter,
    window::ScreenDimensions,
};

/// Resource holding the side length of a tile.
#[derive(Default)]
pub struct TileDimension(pub f32);

#[derive(Copy, Clone, PartialEq)]
pub enum RunState {
    Running,
    Paused,
}

impl Default for RunState {
    fn default() -> Self {
        RunState::Running
    }
}

#[derive(Default)]
pub struct GameState<'a, 'b> {
    running_dispatcher: Option<Dispatcher<'a, 'b>>,
    paused_dispatcher: Option<Dispatcher<'a, 'b>>,

    fps_display: Option<Entity>,
}

impl<'a, 'b> SimpleState for GameState<'a, 'b> {
    fn on_start(&mut self, data: StateData<'_, GameData>) {
        let StateData { world, .. } = data;

        // Create system dispatcher for the running state
        let mut running_dispatcher = DispatcherBuilder::new()
            .with_pool((*world.read_resource::<ArcThreadPool>()).clone())
            .with(VisibilitySystem, "visibility_system", &[])
            .with(MonsterAI, "monster_ai_system", &[])
            .with_barrier()
            .with(MapIndexingSystem, "map_indexing_system", &[])
            .with(PositionTranslator, "position_translator", &[])
            .build();

        // Create system dispatcher for the paused state
        let mut paused_dispatcher = DispatcherBuilder::new()
            .with_pool((*world.read_resource::<ArcThreadPool>()).clone())
            .with(InputDispatcher::default(), "player_movement_system", &[])
            .with_barrier()
            .with(MapIndexingSystem, "map_indexing_system", &[])
            .build();

        // Attach the dispatchers to the world
        running_dispatcher.setup(world);
        paused_dispatcher.setup(world);

        // Store the dispatchers in the state
        self.running_dispatcher = Some(running_dispatcher);
        self.paused_dispatcher = Some(paused_dispatcher);

        let (screen_width, screen_height) = {
            let dim = world.read_resource::<ScreenDimensions>();
            (dim.width(), dim.height())
        };

        let sprite_sheet =
            load_sprite_sheet(world, "texture/cp437_20x20.png", "texture/cp437_20x20.ron");

        // Create required resources
        world.insert(TileDimension(20.0));

        // Initialize world map and camera
        // IMPORTANT: map initialization *must* come before everything else
        create_map(world, 80, 50, sprite_sheet.clone());
        create_camera(world, screen_width, screen_height);

        // Initialize all the game-related entities
        spawn_player(world, sprite_sheet.clone());
        spawn_monsters(world, sprite_sheet.clone());

        // Utilities
        self.create_fps_display(world);
    }

    fn update(&mut self, data: &mut StateData<'_, GameData>) -> SimpleTrans {
        let StateData { world, .. } = data;

        let run_state = *world.read_resource::<RunState>();
        match run_state {
            RunState::Running => {
                // Dispatch game logic systems
                if let Some(ref mut d) = self.running_dispatcher {
                    d.dispatch(&world);
                    *world.write_resource() = RunState::Paused;
                }
            }
            RunState::Paused => {
                // Dispatch input handling systems
                if let Some(ref mut d) = self.paused_dispatcher {
                    d.dispatch(&world);
                }
            }
        }

        self.update_fps_display(world);

        Trans::None
    }
}

impl<'a, 'b> GameState<'a, 'b> {
    // Displays an FPS counter on the top left of the screen.
    fn create_fps_display(&mut self, world: &mut World) {
        let font = world.read_resource::<Loader>().load(
            "font/square.ttf",
            TtfFormat,
            (),
            &world.read_resource(),
        );

        let transform = UiTransform::new(
            "fps".to_string(),
            Anchor::TopLeft,
            Anchor::TopLeft,
            10.0,
            -20.0,
            0.5,
            50.,
            30.,
        );

        self.fps_display = Some(
            world
                .create_entity()
                .with(transform)
                .with(UiText::new(
                    font,
                    "--".to_string(),
                    [0.8, 0.8, 0.0, 1.0],
                    32.,
                ))
                .build(),
        );
    }

    // Updates the FPS counter with the measured FPS.
    fn update_fps_display(&mut self, world: &mut World) {
        let mut ui_text = world.write_storage::<UiText>();
        if let Some(display) = self.fps_display.and_then(|e| ui_text.get_mut(e)) {
            if world.read_resource::<Time>().frame_number() % 20 == 0 {
                let fps = world.read_resource::<FpsCounter>().sampled_fps();
                display.text = format!("{:.0}", fps.round());
            }
        }
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

fn create_camera(world: &mut World, screen_width: f32, screen_height: f32) {
    let tile_dim = world.read_resource::<TileDimension>().0;

    let mut position = Transform::default();
    position.set_translation_xyz(
        (screen_width - tile_dim) / 2.0,
        (screen_height - tile_dim) / 2.0,
        10.0,
    );

    world
        .create_entity()
        .with(position)
        .with(Camera::standard_2d(screen_width, screen_height))
        .build();
}

fn spawn_player(world: &mut World, sheet: Handle<SpriteSheet>) {
    let pos = world.read_resource::<WorldMap>().rooms()[0].center();

    // Insert player position as resource
    world.insert(pos);

    world
        .create_entity()
        .with(Player)
        .with(InputListener)
        .with(Position(pos))
        .with(Viewshed::new(8))
        .with(SpriteRender {
            sprite_sheet: sheet,
            sprite_number: utils::to_glyph('@'),
        })
        .with(Name("Hero".to_string()))
        .with(Tint(Srgba::new(0.7, 0.5, 0.0, 1.0)))
        .build();
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
            .with(Monster)
            .with(Position(spawn_point))
            .with(BlocksTile)
            .with(Viewshed::new(8))
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
