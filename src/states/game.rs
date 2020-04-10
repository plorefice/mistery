//! This module contains the core of the game logic which does not fit into any ECS category.
//! This include the game initialization, map structure, entity spawning logic etc.

use crate::{
    components::*,
    core::{map::WorldMap, spawn},
    graphics::{
        renderer::{self, ConsoleTileMap},
        Ui,
    },
    math::{Point, Rect},
    resources::{CombatLog, TileDimension},
    states::{GameState, GameStateEvent, GameTrans},
    systems::*,
};

use amethyst::{
    assets::{AssetStorage, Handle, Loader},
    core::{math::Vector3, transform::Transform, Parent},
    ecs::{Dispatcher, DispatcherBuilder, Entity},
    input::{is_close_requested, InputEvent},
    prelude::*,
    renderer::{Camera, ImageFormat, SpriteSheet, SpriteSheetFormat, Texture},
    window::ScreenDimensions,
};
use rand::Rng;

const CONSOLE_WIDTH: u32 = 80;
const CONSOLE_HEIGHT: u32 = 50;

const MAP_WIDTH: u32 = 80;
const MAP_HEIGHT: u32 = 50;

/// This is the core game state. This is were the magic happens.
#[derive(Default)]
pub struct RunState<'a, 'b> {
    ui: Option<Ui>,
    console: Option<Entity>,
    input: RunStateInputDispatcher,
    dispatcher: Option<Dispatcher<'a, 'b>>,
}

impl<'a, 'b> GameState for RunState<'a, 'b> {
    fn on_start(&mut self, StateData { world, .. }: StateData<'_, GameData>) {
        // Setup systems for this state
        let mut dispatcher = DispatcherBuilder::new()
            .with(MapIndexingSystem, "map_indexing", &[])
            .with(VisibilitySystem, "visibility", &[])
            .with(MonsterAI, "monster_ai", &["visibility"])
            .with(
                MoveResolver,
                "move_resolver",
                &["monster_ai", "map_indexing"],
            )
            .with(PickUpSystem, "pick_up", &["move_resolver"])
            .with(ItemUsageResolver, "item_usage_resolver", &["move_resolver"])
            .with(ItemDropResolver, "item_drop_resolver", &["move_resolver"])
            .with(MeleeCombatResolver, "melee_resolver", &["move_resolver"])
            .with(DamageResolver, "damage_resolver", &["melee_resolver"])
            .with(
                PositionTranslator,
                "position_translator",
                &["move_resolver"],
            )
            .with(TurnSystem::default(), "turn", &["position_translator"])
            .build();

        dispatcher.setup(world);

        self.dispatcher = Some(dispatcher);

        // Create required resources
        world.insert(TileDimension(20));
        world.insert({
            let mut log = CombatLog::default();
            log.push("Welcome to Mistery!");
            log
        });

        // Register components that are not used in any system.
        world.register::<Pickable>();
        world.register::<Ranged>();
        world.register::<InflictsDamage>();

        // Load spritesheet
        let sprite_sheet =
            load_sprite_sheet(world, "texture/cp437_20x20.png", "texture/cp437_20x20.ron");

        // Initialize world map (*must* come before everything else)
        world.insert(WorldMap::rooms_and_corridors(MAP_WIDTH, MAP_HEIGHT));

        // Initialize all the game-related entities
        let player = spawn_entities(world, sprite_sheet.clone());

        // Allocate console tilemap for rendering
        let console = create_console(world, player, sprite_sheet);

        // Load UI
        self.ui = Some(Ui::new(console));
        self.console = Some(console);
    }

    fn handle_event(
        &mut self,
        StateData { world, .. }: StateData<'_, GameData>,
        event: GameStateEvent,
    ) -> GameTrans {
        match &event {
            StateEvent::Window(event) if is_close_requested(&event) => Trans::Quit,
            StateEvent::Input(InputEvent::ActionPressed(action)) => {
                self.input.handle(world, self.console.unwrap(), *action)
            }
            _ => Trans::None,
        }
    }

    fn update(&mut self, StateData { world, .. }: &mut StateData<'_, GameData>) -> GameTrans {
        if let Some(dispatcher) = &mut self.dispatcher {
            dispatcher.dispatch(world);
        }
        Trans::None
    }

    fn shadow_update(&mut self, StateData { world, .. }: StateData<'_, GameData>) {
        if let Some(map) = self.console {
            renderer::refresh_map_view(world, map);
        }
        if let Some(ui) = &mut self.ui {
            ui.refresh(world);
        }
    }
}

// Allocates a `TileMap` for the console emulation.
fn create_console(world: &mut World, pivot: Entity, sheet: Handle<SpriteSheet>) -> Entity {
    let tile_dim = world.read_resource::<TileDimension>().0;

    let tilemap = ConsoleTileMap::new(
        Vector3::new(CONSOLE_WIDTH, CONSOLE_HEIGHT, 1),
        Vector3::new(tile_dim, tile_dim, 1),
        Some(sheet),
    );

    // Align tilemap to pivot
    let mut transform = Transform::default();
    transform.set_translation_xyz(0., -(tile_dim as f32), 0.);

    world
        .create_entity()
        .with(Parent::new(pivot))
        .with(transform)
        .with(tilemap)
        .build()
}

// Spawns the player, the monsters and the camera. Returns the player entity.
fn spawn_entities(world: &mut World, sheet: Handle<SpriteSheet>) -> Entity {
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
        spawn_room(world, room, sheet.clone());
    }

    // Finally, create the camera
    let (screen_width, screen_height) = {
        let dim = world.read_resource::<ScreenDimensions>();
        (dim.width(), dim.height())
    };
    spawn_camera(world, player, screen_width, screen_height);

    player
}

// Spawns random entities in a room. This includes monsters and items.
fn spawn_room(world: &mut World, room: Rect, sheet: Handle<SpriteSheet>) {
    let mut rng = rand::thread_rng();

    let n_monsters = rng.gen_range(0, spawn::MAX_MONSTERS + 1);
    let n_items = rng.gen_range(0, spawn::MAX_ITEMS + 1);

    // Compute spawn points for both items and monsters
    let mut spawn_points = Vec::with_capacity(n_monsters + n_items);
    for _ in 0..spawn_points.capacity() {
        loop {
            let x = rng.gen_range(room.left() + 1, room.right());
            let y = rng.gen_range(room.bottom() + 1, room.top());
            let pt = Point::new(x, y);

            if !spawn_points.contains(&pt) {
                spawn_points.push(pt);
                break;
            }
        }
    }

    let (monster_spawns, item_spawns) = spawn_points.split_at(n_monsters);

    // Spawn monsters
    for pt in monster_spawns {
        spawn::random_monster(world, *pt, sheet.clone());
    }

    // Spawn items
    for pt in item_spawns {
        spawn::random_item(world, *pt, sheet.clone());
    }
}

// Creates an orthographic camera covering the entire screen view.
fn spawn_camera(world: &mut World, pivot: Entity, screen_width: f32, screen_height: f32) {
    let tile_dim = world.read_resource::<TileDimension>().0 as f32;

    // Put the camera 10 units away from the console, and center it on the pivot
    let mut transform = Transform::default();
    transform.set_translation_xyz(-tile_dim / 2., -tile_dim / 2., 10.);

    world
        .create_entity()
        .with(Parent::new(pivot))
        .with(transform)
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
