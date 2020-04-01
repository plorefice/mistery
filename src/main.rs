mod components;
mod game;
mod input;
mod map;
mod math;
mod renderer;
mod systems;
mod utils;

use game::GameState;
use input::GameBindings;
use renderer::*;

use amethyst::{
    core::transform::TransformBundle,
    input::InputBundle,
    prelude::*,
    renderer::{
        plugins::{RenderFlat2D, RenderToWindow},
        types::DefaultBackend,
        RenderingBundle,
    },
    tiles::{MortonEncoder, RenderTiles2D},
    utils::{application_root_dir, fps_counter::FpsCounterBundle},
};

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());

    let app_root = application_root_dir()?;

    let assets_dir = app_root.join("assets");
    let config_dir = app_root.join("config");

    let display_config_path = config_dir.join("display.ron");
    let bindings_config_path = config_dir.join("bindings.ron");

    let game_data = GameDataBuilder::default()
        // Built-in system bundles
        .with_bundle(FpsCounterBundle::default())?
        .with_bundle(TransformBundle::new())?
        .with_bundle(
            InputBundle::<GameBindings>::new().with_bindings_from_file(bindings_config_path)?,
        )?
        .with_bundle(
            RenderingBundle::<DefaultBackend>::new()
                .with_plugin(
                    RenderToWindow::from_config_path(display_config_path)?
                        .with_clear([0.0, 0.0, 0.0, 0.0]),
                )
                .with_plugin(RenderFlat2D::default())
                .with_plugin(RenderTiles2D::<WorldTile, MortonEncoder>::default()),
        )?;

    let mut game = Application::new(assets_dir, GameState::default(), game_data)?;
    game.run();

    Ok(())
}
