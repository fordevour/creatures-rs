//! Курсор над пространством: экранные координаты -> клетка.

use crate::constants::SPACE_SIZE;
use crate::render::PIXELS_PER_CELL;

/// Клетка пространства под курсором.
///
/// Позиция курсора задаётся в физических пикселях окна; клетка считается
/// делением на размер клетки в пикселях. За пределами пространства
/// координаты прижимаются к краям.
pub fn cell_at(cursor: (f64, f64)) -> (usize, usize) {
    let x = (cursor.0 / PIXELS_PER_CELL as f64).floor().max(0.0) as usize;
    let y = (cursor.1 / PIXELS_PER_CELL as f64).floor().max(0.0) as usize;

    (x.min(SPACE_SIZE - 1), y.min(SPACE_SIZE - 1))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Пиксель курсора переводится в клетку пространства.
    #[test]
    fn cursor_maps_to_cell() {
        assert_eq!(cell_at((0.0, 0.0)), (0, 0));
        assert_eq!(cell_at((3.0, 7.0)), (0, 1));
        assert_eq!(cell_at((4.0, 4.0)), (1, 1));
        assert_eq!(cell_at((100.0, 200.0)), (25, 50));
    }

    /// Курсор за пределами пространства не выходит за его границы.
    #[test]
    fn cursor_is_clamped() {
        let (x, y) = cell_at((100_000.0, -50.0));

        assert_eq!(x, SPACE_SIZE - 1);
        assert_eq!(y, 0);
    }
}
