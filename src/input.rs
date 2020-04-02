use amethyst::input::BindingTypes;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct AxisBindings;

impl fmt::Display for AxisBindings {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActionBinding {
    N,
    W,
    S,
    E,
    NW,
    SW,
    SE,
    NE,
}

impl fmt::Display for ActionBinding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Default, Debug)]
pub struct GameBindings;

impl BindingTypes for GameBindings {
    type Axis = AxisBindings;
    type Action = ActionBinding;
}
