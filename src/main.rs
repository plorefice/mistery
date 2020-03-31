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
use systems::{ai::*, *};

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
    utils::application_root_dir,
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
        .with_bundle(TransformBundle::new())?
        .with_bundle(
            InputBundle::<GameBindings>::new().with_bindings_from_file(bindings_config_path)?,
        )?
        // Custom systems
        .with(InputDispatcher::default(), "player_movement_system", &[])
        .with(VisibilitySystem, "visibility_system", &[])
        .with(MonsterAI, "monster_ai_system", &[])
        // Every other subsystem must be done executing
        // before we perform the Position -> Transform translation!
        .with_barrier()
        .with(PositionTranslator, "position_translator", &[])
        // Rendering must be executed last, after the translator has done its job.
        .with_barrier()
        .with_bundle(
            RenderingBundle::<DefaultBackend>::new()
                .with_plugin(
                    RenderToWindow::from_config_path(display_config_path)?
                        .with_clear([0.0, 0.0, 0.0, 0.0]),
                )
                .with_plugin(RenderFlat2D::default())
                .with_plugin(RenderTiles2D::<WorldTile, MortonEncoder>::default()),
        )?;

    let mut game = Application::new(assets_dir, GameState, game_data)?;
    game.run();

    Ok(())
}
