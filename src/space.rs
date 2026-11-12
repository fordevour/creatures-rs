//! Пространство: три сетки полей и методы доступа к клеткам.

use crate::constants::SPACE_SIZE;

/// Пространство
/// Сверху-вниз     - уменьшается кислород
/// Слева-направо   - уменьшается свет (+ немного света сверху-вниз)
/// Снизу вверх     - уменьшается количество минералов
/// Правило уменьшения концентрации - геометрическая прогрессия.
/// Т.е. на первой клетке от края 1.0, на второй коэффициент затухания,
/// на третьей его квадрат и так далее.
pub struct Space {
    /// Кислород в каждой клетке.
    pub grid_o2: Vec<f32>,
    /// Свет в каждой клетке.
    pub grid_light: Vec<f32>,
    /// Минералы в каждой клетке.
    pub grid_minerals: Vec<f32>,
}

impl Space {
    /// Пустое пространство: все поля равны нулю.
    pub fn empty() -> Self {
        let len = SPACE_SIZE * SPACE_SIZE;

        Space {
            grid_o2: vec![0.0; len],
            grid_light: vec![0.0; len],
            grid_minerals: vec![0.0; len],
        }
    }

    /// Индекс клетки в плоской сетке.
    pub fn index(x: usize, y: usize) -> usize {
        y * SPACE_SIZE + x
    }

    /// Кислород в клетке (0.0, если клетки нет).
    pub fn o2_at(&self, x: usize, y: usize) -> f32 {
        value_at(&self.grid_o2, Self::index(x, y))
    }

    /// Свет в клетке (0.0, если клетки нет).
    pub fn light_at(&self, x: usize, y: usize) -> f32 {
        value_at(&self.grid_light, Self::index(x, y))
    }

    /// Минералы в клетке (0.0, если клетки нет).
    pub fn minerals_at(&self, x: usize, y: usize) -> f32 {
        value_at(&self.grid_minerals, Self::index(x, y))
    }
}

/// Значение сетки по индексу; если сетка короче — считаем его нулевым.
#[inline]
fn value_at(grid: &[f32], index: usize) -> f32 {
    grid.get(index).copied().unwrap_or(0.0)
}
