//! This module contains all the resources used by the ECS.

/// Resource holding the side length of a tile.
#[derive(Default)]
pub struct TileDimension(pub f32);

/// Resource holding the combat log.
#[derive(Default)]
pub struct CombatLog(Vec<String>);

impl CombatLog {
    /// Adds a line to the combat log.
    pub fn push<S: ToString>(&mut self, line: S) {
        self.0.push(line.to_string());
    }

    /// Gets the content of the log.
    pub fn lines(&self) -> &[String] {
        &self.0
    }
}
