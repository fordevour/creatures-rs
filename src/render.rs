//! Отрисовка пространства в холст пикселей формата `0x00RRGGBB`.
//!
//! Одна клетка пространства рисуется блоком `PIXELS_PER_CELL x PIXELS_PER_CELL`
//! пикселей.

use crate::canvas::Canvas;
use crate::cell_color::cell_color;
use crate::constants::{SPACE_SIZE, SPACE_SUB_SIZE};
use crate::space::Space;

/// Пикселей в одной стороне клетки пространства.
///
/// Суб-клетки укладываются квадратом: `SPACE_SUB_SIZE = 16` → `4 x 4`.
pub const PIXELS_PER_CELL: u32 = 4;

/// Размер окна в пикселях: пространство целиком, по `PIXELS_PER_CELL` на клетку.
pub const WINDOW_SIZE: u32 = SPACE_SIZE as u32 * PIXELS_PER_CELL;

/// Проверка, что суб-клетки действительно квадратные.
const _: () = assert!(SPACE_SUB_SIZE == PIXELS_PER_CELL as usize * PIXELS_PER_CELL as usize);

/// Рисует пространство в холст.
///
/// Размер холста — `width x height` пикселей; сетка пространства
/// (`SPACE_SIZE x SPACE_SIZE` клеток) растягивается на него целиком.
pub fn render(space: &Space, width: u32, height: u32) -> Canvas {
    let mut canvas = Canvas::new(width, height);
    let side = SPACE_SIZE as u32;

    for cy in 0..SPACE_SIZE {
        let y0 = (cy as u32 * height / side) as i32;
        let y1 = ((cy as u32 + 1) * height / side) as i32;

        for cx in 0..SPACE_SIZE {
            let x0 = (cx as u32 * width / side) as i32;
            let x1 = ((cx as u32 + 1) * width / side) as i32;

            let color = cell_color(space, cx, cy);

            canvas.fill_rect(x0, y0, (x1 - x0) as u32, (y1 - y0) as u32, color);
        }
    }

    canvas
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::space_gradients;

    /// Размер кадра совпадает с размером окна.
    #[test]
    fn frame_has_window_size() {
        let space = space_gradients::gradients();
        let canvas = render(&space, WINDOW_SIZE, WINDOW_SIZE);

        assert_eq!(canvas.pixels().len(), (WINDOW_SIZE * WINDOW_SIZE) as usize);
        assert_eq!(canvas.width(), WINDOW_SIZE);
        assert_eq!(canvas.height(), WINDOW_SIZE);
    }

    /// Клетка рисуется блоком `PIXELS_PER_CELL x PIXELS_PER_CELL` пикселей.
    #[test]
    fn cell_is_drawn_as_block() {
        let space = space_gradients::gradients();
        let canvas = render(&space, WINDOW_SIZE, WINDOW_SIZE);
        let pixels = canvas.pixels();

        let first = pixels[0];
        let inside_cell = pixels[PIXELS_PER_CELL as usize - 1];

        // Внутри клетки цвет одинаковый...
        assert_eq!(first, inside_cell);

        // ...а у далёкой клетки — другой.
        let side = WINDOW_SIZE as usize;
        let far_cell = (side - 1) * side + (side - 1);

        assert_ne!(first, pixels[far_cell]);
    }

    /// Градиенты отличают пространство от белого фона.
    #[test]
    fn gradients_are_visible() {
        let space = space_gradients::gradients();
        let canvas = render(&space, WINDOW_SIZE, WINDOW_SIZE);
        let pixels = canvas.pixels();

        let white = (255u32 << 16) | (255 << 8) | 255;
        let side = WINDOW_SIZE as usize;

        // Левый верхний угол: свет (слева) и кислород (сверху) — не белый.
        assert_ne!(pixels[0], white);

        // Свет гаснет слева направо: левый верхний угол != правый верхний.
        assert_ne!(pixels[0], pixels[side - 1]);

        // Кислород сверху, минералы снизу: верхний левый != нижний левый.
        assert_ne!(pixels[0], pixels[(side - 1) * side]);
    }
}
