use crate::components::*;

use amethyst::{
    assets::{AssetStorage, Handle, Loader},
    core::transform::Transform,
    ecs::Entity,
    prelude::*,
    renderer::{
        palette::Srgba, resources::Tint, Camera, ImageFormat, SpriteRender, SpriteSheet,
        SpriteSheetFormat, Texture,
    },
    window::ScreenDimensions,
};

pub struct GameState;

impl SimpleState for GameState {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let StateData { world, .. } = data;

        let (width, height) = {
            let dim = world.read_resource::<ScreenDimensions>();
            (dim.width(), dim.height())
        };

        let cp437_sheet_handle =
            load_sprite_sheet(world, "texture/cp437_20x20.png", "texture/cp437_20x20.ron");

        let player = spawn_entity(
            world,
            [40, 25],
            64,
            Srgba::new(1.0, 1.0, 0.0, 1.0),
            cp437_sheet_handle.clone(),
        );

        world.write_storage().insert(player, InputListener).unwrap();

        initialize_camera(world, width, height);
    }

    fn update(&mut self, _data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        Trans::None
    }
}

fn spawn_entity(
    world: &mut World,
    pos: [i32; 2],
    glyph: usize,
    color: Srgba,
    sheet: Handle<SpriteSheet>,
) -> Entity {
    world
        .create_entity()
        .with(Position(pos.into()))
        .with(Transform::default())
        .with(SpriteRender {
            sprite_sheet: sheet,
            sprite_number: glyph,
        })
        .with(Tint(color))
        .build()
}

fn initialize_camera(world: &mut World, width: f32, height: f32) {
    let mut position = Transform::default();
    position.set_translation_xyz(width / 2.0, height / 2.0, 1.0);

    world
        .create_entity()
        .with(position)
        .with(Camera::standard_2d(width, height))
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
