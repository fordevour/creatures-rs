//! Инфо-панель: значения полей в клетке, на которую наведён курсор.
//!
//! Панель рисуется в правом верхнем углу окна поверх пространства.

use crate::canvas::Canvas;
use crate::cursor::cell_at;
use crate::font::{text_height, text_width};
use crate::space::Space;
use crate::text::draw_text;
use crate::value::format_value;

/// Отступ панели от края окна.
const MARGIN: i32 = 8;

/// Внутренний отступ панели.
const PADDING: i32 = 8;

/// Масштаб шрифта панели.
const TEXT_SCALE: u32 = 2;

/// Высота строки текста панели.
const LINE_HEIGHT: i32 = text_height(TEXT_SCALE) as i32 + 4;

/// Ширина колонки с названиями полей.
const LABEL_WIDTH: usize = 9;

/// Цвет фона панели.
const BACKGROUND: u32 = 0xF2F2F2;

/// Цвет рамки панели.
const BORDER: u32 = 0x303030;

/// Цвет обычного текста.
const TEXT: u32 = 0x202020;

/// Цвет заголовка панели.
const TITLE: u32 = 0x8A4B08;

/// Рисует панель в правом верхнем углу холста.
///
/// `cursor` — позиция курсора в физических пикселях окна.
pub fn draw(canvas: &mut Canvas, space: &Space, cursor: (f64, f64)) {
    let (cell_x, cell_y) = cell_at(cursor);

    let lines = [
        (format!("CELL {cell_x},{cell_y}"), TITLE),
        (
            format!(
                "{:<LABEL_WIDTH$}{}",
                "O2",
                format_value(space.o2_at(cell_x, cell_y))
            ),
            TEXT,
        ),
        (
            format!(
                "{:<LABEL_WIDTH$}{}",
                "LIGHT",
                format_value(space.light_at(cell_x, cell_y))
            ),
            TEXT,
        ),
        (
            format!(
                "{:<LABEL_WIDTH$}{}",
                "MINERALS",
                format_value(space.minerals_at(cell_x, cell_y))
            ),
            TEXT,
        ),
    ];

    let text = lines
        .iter()
        .map(|(text, _)| text_width(text, TEXT_SCALE))
        .max()
        .unwrap_or(0) as i32;

    let panel_width = text + 2 * PADDING;
    let panel_height = lines.len() as i32 * LINE_HEIGHT + 2 * PADDING - 4;

    let x = canvas.width() as i32 - MARGIN - panel_width;
    let y = MARGIN;

    canvas.fill_rect(x, y, panel_width as u32, panel_height as u32, BACKGROUND);
    canvas.draw_rect(x, y, panel_width as u32, panel_height as u32, BORDER);

    let mut text_y = y + PADDING;

    for (text, color) in &lines {
        draw_text(canvas, x + PADDING, text_y, text, *color, TEXT_SCALE);
        text_y += LINE_HEIGHT;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::{WINDOW_SIZE, render};
    use crate::space_gradients;

    /// Панель рисуется в правом верхнем углу поверх пространства.
    #[test]
    fn panel_is_drawn_in_top_right_corner() {
        let space = space_gradients::gradients();
        let mut canvas = render(&space, WINDOW_SIZE, WINDOW_SIZE);
        let before = canvas.pixels().to_vec();

        draw(&mut canvas, &space, (100.0, 200.0));

        let after = canvas.pixels();
        let side = WINDOW_SIZE as usize;

        // Пиксель внутри панели (правый верхний угол) меняется.
        let inside_panel = (MARGIN as usize + 2) * side + (side - 20);
        assert_ne!(after[inside_panel], before[inside_panel]);

        // Левый нижний угол панель не задевает.
        assert_eq!(after[(side - 1) * side], before[(side - 1) * side]);
    }
}
