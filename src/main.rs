mod components;
mod core;
mod math;
mod renderer;
mod resources;
mod states;
mod systems;
mod ui;
mod utils;

use crate::{renderer::*, states::GameState, systems::*};

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
    ui::{RenderUi, UiBundle},
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
        .with_bundle(
            InputBundle::<GameBindings>::new().with_bindings_from_file(bindings_config_path)?,
        )?
        .with_bundle(TransformBundle::new())?
        .with_bundle(FpsCounterBundle::default())?
        .with_bundle(UiBundle::<GameBindings>::new())?
        .with_bundle(
            RenderingBundle::<DefaultBackend>::new()
                .with_plugin(
                    RenderToWindow::from_config_path(display_config_path)?
                        .with_clear([0.0, 0.0, 0.0, 0.0]),
                )
                .with_plugin(RenderUi::default())
                .with_plugin(RenderFlat2D::default())
                .with_plugin(RenderTiles2D::<WorldTile, MortonEncoder>::default()),
        )?
        // Game logic
        .with(MapIndexingSystem, "map_indexing", &[])
        .with(VisibilitySystem, "visibility", &[])
        .with(TurnSystem::default(), "turn", &[])
        .with(
            InputDispatcher::default(),
            "player_movement",
            &["visibility", "turn"],
        )
        .with(MonsterAI, "monster_ai", &["visibility", "turn"])
        .with(
            MoveResolver,
            "move_resolver",
            &["player_movement", "monster_ai", "map_indexing"],
        )
        .with(PickUpSystem, "pick_up", &["move_resolver"])
        .with(MeleeCombatResolver, "melee_resolver", &["move_resolver"])
        .with(DamageResolver, "damage_resolver", &["melee_resolver"])
        .with(
            PositionTranslator,
            "position_translator",
            &["move_resolver"],
        );

    let mut game = Application::new(assets_dir, GameState::default(), game_data)?;
    game.run();

    Ok(())
}
