//! Отрисовка пространства в холст пикселей формата `0x00RRGGBB`.
//!
//! Каждая клетка рисуется блоком, а по её границам проходит еле-видная
//! рамка, чтобы было понятно, где клетки, а где просто пространство.

use crate::camera::Camera;
use crate::canvas::Canvas;
use crate::cell_color::cell_border_color;
use crate::cell_colors::CellColors;
use crate::constants::{SPACE_SIZE, SPACE_SUB_SIZE};
use crate::palette::BACKGROUND_COLOR;

/// Пикселей в одной стороне клетки пространства.
///
/// Суб-клетки укладываются квадратом: `SPACE_SUB_SIZE = 16` → `4 x 4`.
pub const PIXELS_PER_CELL: u32 = 4;

/// Размер окна в пикселях: пространство целиком, по `PIXELS_PER_CELL` на клетку.
pub const WINDOW_SIZE: u32 = SPACE_SIZE as u32 * PIXELS_PER_CELL;

/// Проверка, что суб-клетки действительно квадратные.
const _: () = assert!(SPACE_SUB_SIZE == PIXELS_PER_CELL as usize * PIXELS_PER_CELL as usize);

/// Минимальный размер клетки в пикселях, при котором рисуется рамка.
const MIN_BORDER_CELL: f32 = 2.0;

/// Рисует пространство в холст.
///
/// Видно ту часть пространства, которую показывает камера. Холст
/// переиспользуется между кадрами, поэтому его размер должен быть задан заранее.
pub fn render(camera: &Camera, colors: &CellColors, canvas: &mut Canvas) {
    let width = canvas.width();
    let height = canvas.height();

    canvas.fill(BACKGROUND_COLOR);

    let (min_x, max_x, min_y, max_y) = camera.visible_cells(width, height);
    let with_borders = camera.pixels_per_cell() >= MIN_BORDER_CELL;

    for cy in min_y..=max_y {
        for cx in min_x..=max_x {
            let color = colors.get(cx, cy);

            let (x0, y0) = camera.cell_origin(cx as f32, cy as f32, width, height);
            let (x1, y1) = camera.cell_origin(cx as f32 + 1.0, cy as f32 + 1.0, width, height);

            let cell_width = (x1 - x0).max(1) as u32;
            let cell_height = (y1 - y0).max(1) as u32;

            canvas.fill_rect(x0, y0, cell_width, cell_height, color);

            // Рамка идёт по правой и нижней границе клетки, поэтому соседняя
            // клетка её не затирает.
            if with_borders {
                let border = cell_border_color(color);

                canvas.fill_rect(x0 + cell_width as i32 - 1, y0, 1, cell_height, border);
                canvas.fill_rect(x0, y0 + cell_height as i32 - 1, cell_width, 1, border);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::space_gradients;

    /// Готовит кэш цветов и холст для теста.
    fn scene() -> (Camera, CellColors, Canvas) {
        let space = space_gradients::gradients();
        let mut colors = CellColors::new();
        colors.rebuild(&space);

        (Camera::new(), colors, Canvas::new(WINDOW_SIZE, WINDOW_SIZE))
    }

    /// Холст после отрисовки совпадает с размером окна.
    #[test]
    fn frame_has_window_size() {
        let (camera, colors, mut canvas) = scene();

        render(&camera, &colors, &mut canvas);

        assert_eq!(canvas.pixels().len(), (WINDOW_SIZE * WINDOW_SIZE) as usize);
        assert_eq!(canvas.width(), WINDOW_SIZE);
        assert_eq!(canvas.height(), WINDOW_SIZE);
    }

    /// Клетка рисуется блоком `PIXELS_PER_CELL x PIXELS_PER_CELL` пикселей.
    #[test]
    fn cell_is_drawn_as_block() {
        let (camera, colors, mut canvas) = scene();

        render(&camera, &colors, &mut canvas);

        let pixels = canvas.pixels();

        // Внутри клетки (без рамки) цвет одинаковый...
        let inside = pixels[1];
        assert_eq!(inside, pixels[2]);

        // ...а у далёкой клетки — другой.
        let side = WINDOW_SIZE as usize;
        let far_cell = (side - 1) * side + (side - 1);

        assert_ne!(pixels[0], pixels[far_cell]);
    }

    /// Градиенты отличают пространство от белого фона.
    #[test]
    fn gradients_are_visible() {
        let (camera, colors, mut canvas) = scene();

        render(&camera, &colors, &mut canvas);

        let pixels = canvas.pixels();
        let side = WINDOW_SIZE as usize;

        // Свет гаснет слева направо: левый верхний угол != правый верхний.
        assert_ne!(pixels[0], pixels[side - 1]);

        // Кислород сверху, минералы снизу: верхний левый != нижний левый.
        assert_ne!(pixels[0], pixels[(side - 1) * side]);
    }

    /// По границам клеток проходит еле-видная рамка.
    #[test]
    fn cells_have_borders() {
        let (camera, colors, mut canvas) = scene();

        render(&camera, &colors, &mut canvas);

        let pixels = canvas.pixels();

        // Первые три пикселя — сама клетка, четвёртый — рамка.
        assert_eq!(pixels[0], pixels[2]);
        assert_ne!(pixels[3], pixels[2]);
    }
}
