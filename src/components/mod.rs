use amethyst::{
    core::math::Vector2,
    ecs::{Component, DenseVecStorage},
};

#[derive(Copy, Clone, Debug, Component)]
pub(crate) struct Position(pub Vector2<i32>);

#[derive(Default, Copy, Clone, Debug, Component)]
pub(crate) struct LeftMover;

#[derive(Default, Copy, Clone, Debug, Component)]
pub(crate) struct InputMover;
