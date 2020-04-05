use crate::{components::*, math::Point, utils};

use amethyst::{
    assets::Handle,
    core::Hidden,
    ecs::Entity,
    prelude::*,
    renderer::{palette::Srgba, resources::Tint, SpriteRender, SpriteSheet},
};

/// Maximum number of monsters that can spawn in a room.
pub const MAX_MONSTERS: usize = 4;

/// Maximum number of items that can spawn in a room.
pub const MAX_ITEMS: usize = 2;

/// Spawns the player entity at the given coordinates.
pub fn player(world: &mut World, pos: Point, sheet: Handle<SpriteSheet>) -> Entity {
    // Insert player position as resource
    world.insert(pos);

    world
        .create_entity()
        .with(Player)
        .with(Faction(0))
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

/// Spawns a random monster at the given coordinates.
pub fn random_monster(world: &mut World, pos: Point, sheet: Handle<SpriteSheet>) -> Entity {
    if rand::random() {
        orc(world, pos, sheet)
    } else {
        goblin(world, pos, sheet)
    }
}

/// Spawns an orc at the given coordinates.
pub fn orc(world: &mut World, pos: Point, sheet: Handle<SpriteSheet>) -> Entity {
    monster(world, pos, utils::to_glyph('o'), "Orc", sheet)
}

/// Spawns a goblin at the given coordinates.
pub fn goblin(world: &mut World, pos: Point, sheet: Handle<SpriteSheet>) -> Entity {
    monster(world, pos, utils::to_glyph('g'), "Goblin", sheet)
}

// Spawns a monster at the given coordinates using the specified glyph and name.
fn monster<S: ToString>(
    world: &mut World,
    pos: Point,
    glyph: usize,
    name: S,
    sheet: Handle<SpriteSheet>,
) -> Entity {
    world
        .create_entity()
        .with(Faction(1))
        .with(ActsOnTurns::default())
        .with(Position(pos))
        .with(BlocksTile)
        .with(Viewshed::new(8))
        .with(CombatStats {
            max_hp: 16,
            hp: 16,
            defense: 1,
            power: 4,
        })
        .with(SpriteRender {
            sprite_sheet: sheet,
            sprite_number: glyph,
        })
        .with(Name(name.to_string()))
        .with(Tint(Srgba::new(1.0, 0.0, 0.0, 1.0)))
        .with(Hidden) // initially monsters are not visible
        .build()
}

/// Spawns a health potion at the given coordinates.
pub fn health_potion(world: &mut World, pos: Point, sheet: Handle<SpriteSheet>) -> Entity {
    world
        .create_entity()
        .with(Pickable)
        .with(HealsUser { amount: 8 })
        .with(Position(pos))
        .with(SpriteRender {
            sprite_sheet: sheet,
            sprite_number: utils::to_glyph('¡'),
        })
        .with(Name(String::from("Health Potion")))
        .with(Tint(Srgba::new(1.0, 0.0, 1.0, 1.0)))
        .build()
}
