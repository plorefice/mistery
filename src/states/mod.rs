//! This modules contains all the Amethyst [`State`]s that the game can be in.

mod game;
mod inventory;

// Re-export all modules
pub use game::*;
pub use inventory::*;

use crate::systems::GameBindings;

use amethyst::{
    input::is_close_requested, GameData, State, StateData, StateEvent, StateEventReader, Trans,
};

use std::ops::{Deref, DerefMut};

/// A `StateEvent` which wraps `GameBindings` for input handling.
pub type GameStateEvent = StateEvent<GameBindings>;

/// A `StateEventReader` which wraps `GameBindings` for input handling.
pub type GameStateEventReader = StateEventReader<GameBindings>;

/// A custom `Trans` made to be used with `GameState`.
/// It contains a `GameData` as its `StateData` and uses a custom `GameStateEvent`.
pub type GameTrans = Trans<GameData<'static, 'static>, GameStateEvent>;

/// A `State` trait specific for this game.
/// It contains `GameData` as its `StateData` wraps `GameBindings` in its `StateEvent`.
pub trait GameState {
    /// Executed when the game state begins.
    fn on_start(&mut self, _data: StateData<'_, GameData<'static, 'static>>) {}

    /// Executed when the game state exits.
    fn on_stop(&mut self, _data: StateData<'_, GameData<'static, 'static>>) {}

    /// Executed when a different game state is pushed onto the stack.
    fn on_pause(&mut self, _data: StateData<'_, GameData<'static, 'static>>) {}

    /// Executed when the application returns to this game state once again.
    fn on_resume(&mut self, _data: StateData<'_, GameData<'static, 'static>>) {}

    /// Executed on every frame before updating, for use in reacting to events.
    fn handle_event(
        &mut self,
        _data: StateData<'_, GameData<'static, 'static>>,
        event: GameStateEvent,
    ) -> GameTrans {
        if let StateEvent::Window(event) = &event {
            if is_close_requested(&event) {
                Trans::Quit
            } else {
                Trans::None
            }
        } else {
            Trans::None
        }
    }

    /// Executed repeatedly at stable, predictable intervals (1/60th of a second by default).
    fn fixed_update(&mut self, _data: StateData<'_, GameData<'static, 'static>>) -> GameTrans {
        Trans::None
    }

    /// Executed on every frame immediately, as fast as the engine will allow
    /// (taking into account the frame rate limit).
    fn update(&mut self, _data: &mut StateData<'_, GameData<'static, 'static>>) -> GameTrans {
        Trans::None
    }

    /// Executed repeatedly at stable, predictable intervals (1/60th of a second by default),
    /// even when this is not the active state, as long as this state is on the
    /// [StateMachine](struct.StateMachine.html)'s state-stack.
    fn shadow_fixed_update(&mut self, _data: StateData<'_, GameData<'static, 'static>>) {}

    /// Executed on every frame immediately, as fast as the engine will allow
    /// (taking into account the frame rate limit), even when this is not the active state,
    /// as long as this state is on the [StateMachine](struct.StateMachine.html)'s state-stack.
    fn shadow_update(&mut self, _data: StateData<'_, GameData<'static, 'static>>) {}
}

impl<T: GameState> State<GameData<'static, 'static>, GameStateEvent> for GameStateWrapper<T> {
    /// Executed when the game state begins.
    fn on_start(&mut self, data: StateData<'_, GameData<'static, 'static>>) {
        self.deref_mut().on_start(data)
    }

    /// Executed when the game state exits.
    fn on_stop(&mut self, data: StateData<'_, GameData<'static, 'static>>) {
        self.deref_mut().on_stop(data)
    }

    /// Executed when a different game state is pushed onto the stack.
    fn on_pause(&mut self, data: StateData<'_, GameData<'static, 'static>>) {
        self.deref_mut().on_pause(data)
    }

    /// Executed when the application returns to this game state once again.
    fn on_resume(&mut self, data: StateData<'_, GameData<'static, 'static>>) {
        self.deref_mut().on_resume(data)
    }

    /// Executed on every frame before updating, for use in reacting to events.
    fn handle_event(
        &mut self,
        data: StateData<'_, GameData<'static, 'static>>,
        event: GameStateEvent,
    ) -> GameTrans {
        self.deref_mut().handle_event(data, event)
    }

    /// Executed repeatedly at stable, predictable intervals (1/60th of a second by default).
    fn fixed_update(&mut self, data: StateData<'_, GameData<'static, 'static>>) -> GameTrans {
        self.deref_mut().fixed_update(data)
    }

    /// Executed on every frame immediately, as fast as the engine will allow
    /// (taking into account the frame rate limit).
    fn update(&mut self, mut data: StateData<'_, GameData<'static, 'static>>) -> GameTrans {
        let r = self.deref_mut().update(&mut data);
        data.data.update(&data.world);
        r
    }

    /// Executed repeatedly at stable, predictable intervals (1/60th of a second by default),
    /// even when this is not the active state, as long as this state is on the
    /// [StateMachine](struct.StateMachine.html)'s state-stack.
    fn shadow_fixed_update(&mut self, data: StateData<'_, GameData<'static, 'static>>) {
        self.deref_mut().shadow_fixed_update(data)
    }

    /// Executed on every frame immediately, as fast as the engine will allow
    /// (taking into account the frame rate limit), even when this is not the active state,
    /// as long as this state is on the [StateMachine](struct.StateMachine.html)'s state-stack.
    fn shadow_update(&mut self, data: StateData<'_, GameData<'static, 'static>>) {
        self.deref_mut().shadow_update(data)
    }
}

/// A wrapper type for `GameState` implementors to circumvent the orphan rule.
pub struct GameStateWrapper<T>(T);

impl<T> GameStateWrapper<T> {
    /// Wraps an existing game state.
    pub fn new(state: T) -> GameStateWrapper<T> {
        GameStateWrapper(state)
    }
}

impl<T> Deref for GameStateWrapper<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for GameStateWrapper<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
