use crate::{components::*, map::WorldMap, renderer::WorldTileMap, utils};

use amethyst::{
    assets::{AssetStorage, Handle, Loader},
    core::{
        math::Vector3,
        transform::{Parent, Transform},
        Hidden,
    },
    ecs::{Entity, Join},
    prelude::*,
    renderer::{
        palette::Srgba, resources::Tint, Camera, ImageFormat, SpriteRender, SpriteSheet,
        SpriteSheetFormat, Texture,
    },
    ui::{UiCreator, UiFinder, UiText, UiTransform},
    utils::fps_counter::FpsCounter,
    window::ScreenDimensions,
};
use itertools::Itertools;
use std::collections::HashMap;

/// Resource holding the side length of a tile.
#[derive(Default)]
pub struct TileDimension(pub f32);

/// Resource holding the combat log.
#[derive(Default)]
pub struct CombatLog(Vec<String>);

impl CombatLog {
    /// Adds a line to the combat log.
    pub fn push<S: AsRef<str>>(&mut self, line: S) {
        self.0.push(line.as_ref().to_owned());
    }

    /// Gets the content of the log.
    pub fn lines(&self) -> &[String] {
        &self.0
    }
}

#[derive(Default)]
pub struct GameState {
    ui_cache: HashMap<String, Entity>,
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

        // Draw UI
        world.exec(|mut creator: UiCreator<'_>| {
            creator.create("ui/fps-counter.ron", ());
            creator.create("ui/infobox.ron", ());
        });
    }

    fn update(&mut self, StateData { world, .. }: &mut StateData<'_, GameData>) -> SimpleTrans {
        // Update UI
        self.update_infobox(world);
        self.update_fps_counter(world);

        Trans::None
    }
}

impl GameState {
    // Updates the infobox to reflect the current game state.
    fn update_infobox(&mut self, world: &mut World) {
        self.update_hp_display(world);
        self.update_combat_log(world);
    }

    // Update the HP text and bar in the infobox.
    fn update_hp_display(&mut self, world: &mut World) {
        let hp_text = self.get_ui_element("hp-text", world);
        let hp_slot = self.get_ui_element("hp-slot", world);
        let hp_bar = self.get_ui_element("hp-bar", world);

        if let (Some(hp_text), Some(hp_slot), Some(hp_bar)) = (hp_text, hp_slot, hp_bar) {
            let players = world.read_storage::<Player>();
            let stats = world.read_storage::<CombatStats>();

            let mut ui_text = world.write_storage::<UiText>();
            let mut ui_transform = world.write_storage::<UiTransform>();

            if let Some((_, stats)) = (&players, &stats).join().next() {
                if let Some(hp) = ui_text.get_mut(hp_text) {
                    hp.text = format!("HP: {} / {}", stats.hp, stats.max_hp);
                }

                let ratio = stats.hp as f32 / stats.max_hp as f32;
                let width = if let Some(hp_slot) = ui_transform.get_mut(hp_slot) {
                    hp_slot.width
                } else {
                    return;
                };

                if let Some(hp_bar) = ui_transform.get_mut(hp_bar) {
                    hp_bar.width = width * ratio;
                }
            }
        }
    }

    // Update the combat log to show the most recent messages.
    fn update_combat_log(&mut self, world: &mut World) {
        if let Some(ui_log) = self.get_ui_element("combat-log", world) {
            let mut ui_text = world.write_storage::<UiText>();
            if let Some(ui_log) = ui_text.get_mut(ui_log) {
                ui_log.text = world
                    .read_resource::<CombatLog>()
                    .lines()
                    .iter()
                    .rev()
                    .take(3)
                    .rev()
                    .cloned()
                    .intersperse("\n".to_string())
                    .collect::<String>();
            }
        }
    }

    // Updates the FPS counter with the currently measured FPS.
    fn update_fps_counter(&mut self, world: &mut World) {
        if let Some(fps) = self.get_ui_element("fps-counter", world) {
            let mut ui_text = world.write_storage::<UiText>();
            if let Some(counter) = ui_text.get_mut(fps) {
                counter.text = format!(
                    "{:.0}",
                    world.read_resource::<FpsCounter>().sampled_fps().round()
                );
            }
        }
    }

    /// Returns the entity corresponding to the UI element with the given ID, if it exists.
    fn get_ui_element<S: AsRef<str>>(&mut self, id: S, world: &mut World) -> Option<Entity> {
        match self.ui_cache.get(id.as_ref()) {
            Some(e) => Some(*e),
            None => {
                if let Some(e) = world.exec(|f: UiFinder<'_>| f.find(id.as_ref())) {
                    self.ui_cache.insert(id.as_ref().to_owned(), e)
                } else {
                    None
                }
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
