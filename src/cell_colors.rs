//! Кэш цветов клеток.
//!
//! Цвет клетки зависит только от данных пространства, а не от камеры, поэтому
//! пересчитывается не каждый кадр, а лишь при изменении пространства.

use crate::cell_color::cell_color;
use crate::constants::SPACE_SIZE;
use crate::space::Space;

/// Цвета всех клеток пространства.
pub struct CellColors {
    colors: Vec<u32>,
}

impl CellColors {
    /// Пустой кэш.
    pub fn new() -> Self {
        CellColors { colors: Vec::new() }
    }

    /// Пересчитывает цвета всех клеток пространства.
    pub fn rebuild(&mut self, space: &Space) {
        self.colors.clear();
        self.colors.reserve(SPACE_SIZE * SPACE_SIZE);

        for y in 0..SPACE_SIZE {
            for x in 0..SPACE_SIZE {
                self.colors.push(cell_color(space, x, y));
            }
        }
    }

    /// Цвет клетки (0, если кэша для неё ещё нет).
    pub fn get(&self, x: usize, y: usize) -> u32 {
        self.colors.get(y * SPACE_SIZE + x).copied().unwrap_or(0)
    }
}

impl Default for CellColors {
    fn default() -> Self {
        CellColors::new()
    }
}
