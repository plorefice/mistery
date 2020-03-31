use crate::{components::*, math::Rect, renderer::WorldTileMap};

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
use rand::Rng;

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

        // Create world map
        initialize_map(world, 80, 50, sprite_sheet.clone());

        let pos = world.read_resource::<WorldMap>().rooms[0].center();

        // Create player entity
        world
            .create_entity()
            .with(InputListener)
            .with(Position([pos[0] as i32, pos[1] as i32].into()))
            .with(SpriteRender {
                sprite_sheet,
                sprite_number: 64,
            })
            .with(Tint(Srgba::new(0.7, 0.5, 0.0, 1.0)))
            .build();

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

    world.insert(WorldMap::rooms_and_corridors(width, height));

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

#[derive(Clone, Copy, PartialEq)]
pub enum TileKind {
    Wall,
    Floor,
}

#[derive(Default)]
pub struct WorldMap {
    width: u32,
    height: u32,

    rooms: Vec<Rect>,
    tiles: Vec<TileKind>,
}

impl WorldMap {
    pub fn rooms_and_corridors(width: u32, height: u32) -> WorldMap {
        const MAX_ROOMS: usize = 30;
        const MIN_SIZE: u32 = 7;
        const MAX_SIZE: u32 = 12;

        let mut map = WorldMap {
            width,
            height,
            rooms: Vec::with_capacity(MAX_ROOMS),
            tiles: vec![TileKind::Wall; (width * height) as usize],
        };

        let mut rng = rand::thread_rng();

        for _ in 0..MAX_ROOMS {
            let w = rng.gen_range(MIN_SIZE, MAX_SIZE);
            let h = rng.gen_range(MIN_SIZE, MAX_SIZE);
            let x = rng.gen_range(1, width - w - 1);
            let y = rng.gen_range(1, height - h - 1);

            let r = Rect::new(x, y, w, h);

            if !map.rooms.iter().any(|r2| r.intersects(r2)) {
                map.create_room(&r);

                if let Some(rp) = map.rooms.last() {
                    let (x1, y1) = (rp.center()[0], rp.center()[1]);
                    let (x2, y2) = (r.center()[0], r.center()[1]);

                    if rng.gen::<bool>() {
                        map.create_horizontal_corridor(x1, x2, y1);
                        map.create_vertical_corridor(y1, y2, x2);
                    } else {
                        map.create_vertical_corridor(y1, y2, x1);
                        map.create_horizontal_corridor(x1, x2, y2);
                    }
                }

                map.rooms.push(r);
            }
        }

        map
    }

    /// Returns the map's height, ie. the number of rows.
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Returns the tile at coordinates `(x, y)`, if present
    pub fn get(&self, x: u32, y: u32) -> Option<TileKind> {
        self.tiles.get(self.xy_to_idx(x, y)).map(ToOwned::to_owned)
    }

    fn xy_to_idx(&self, x: u32, y: u32) -> usize {
        (y * self.width + x) as usize
    }

    fn create_room(&mut self, room: &Rect) {
        for y in room.bottom() + 1..room.top() {
            for x in room.left() + 1..room.right() {
                let idx = self.xy_to_idx(x, y);
                self.tiles[idx] = TileKind::Floor
            }
        }
    }

    fn create_horizontal_corridor(&mut self, x1: u32, x2: u32, y: u32) {
        for x in x1.min(x2)..=x1.max(x2) {
            let idx = self.xy_to_idx(x, y);
            self.tiles[idx] = TileKind::Floor;
        }
    }

    fn create_vertical_corridor(&mut self, y1: u32, y2: u32, x: u32) {
        for y in y1.min(y2)..=y1.max(y2) {
            let idx = self.xy_to_idx(x, y);
            self.tiles[idx] = TileKind::Floor;
        }
    }
}
