//! Правая колонка меню: вся информация о пространстве и камере.
//!
//! Колонка занимает фиксированную ширину у правого края окна и тянется
//! во всю высоту. Слева от неё живёт зона пространства.

use crate::canvas::Canvas;
use crate::font::{text_height, text_width};
use crate::menu_data::MenuData;
use crate::text::draw_text;
use crate::value::format_value;

/// Ширина колонки меню в пикселях.
pub const WIDTH: u32 = 200;

/// Внутренний отступ колонки.
const PADDING: i32 = 8;

/// Масштаб шрифта меню.
const TEXT_SCALE: u32 = 2;

/// Высота строки текста.
const LINE_HEIGHT: i32 = text_height(TEXT_SCALE) as i32 + 4;

/// Отступ после разделителя.
const SECTION_GAP: i32 = 4;

/// Цвет фона колонки.
const BACKGROUND: u32 = 0xF2F2F2;

/// Цвет границы между колонкой и пространством.
const BORDER: u32 = 0x303030;

/// Цвет разделителя секций.
const SEPARATOR: u32 = 0xD4D4D4;

/// Цвет обычного текста.
const TEXT: u32 = 0x202020;

/// Цвет названия раздела.
const HEADING: u32 = 0x707070;

/// Цвет заголовка колонки.
const TITLE: u32 = 0x8A4B08;

/// Рисует колонку меню в правой части холста.
pub fn draw(canvas: &mut Canvas, data: &MenuData) {
    let panel_x = canvas.width().saturating_sub(WIDTH);

    // Фон колонки и граница со стороны пространства.
    canvas.fill_rect(panel_x as i32, 0, WIDTH, canvas.height(), BACKGROUND);
    canvas.fill_rect(panel_x as i32, 0, 1, canvas.height(), BORDER);

    let left = panel_x as i32 + PADDING;
    let right = canvas.width() as i32 - PADDING;

    let mut y = PADDING;

    draw_text(canvas, left, y, "CREATURES", TITLE, TEXT_SCALE);
    y += LINE_HEIGHT;

    y = separator(canvas, left, right, y);

    // Клетка под курсором.
    y = heading(canvas, left, y, "CELL");

    match data.cell.cell {
        Some((x, y_cell)) => {
            draw_text(canvas, left, y, &format!("{x},{y_cell}"), TEXT, TEXT_SCALE);
        }
        None => {
            draw_text(canvas, left, y, "-,-", TEXT, TEXT_SCALE);
        }
    }
    y += LINE_HEIGHT;

    y = row(canvas, left, right, y, "O2", &value(data.cell.o2));
    y = row(canvas, left, right, y, "LIGHT", &value(data.cell.light));
    y = row(canvas, left, right, y, "MINERALS", &value(data.cell.minerals));

    y = separator(canvas, left, right, y);

    // Камера.
    y = heading(canvas, left, y, "CAMERA");
    y = row(canvas, left, right, y, "ZOOM", &format!("{:.1}X", data.zoom));
    y = row(
        canvas,
        left,
        right,
        y,
        "CENTER",
        &format!("{:.0},{:.0}", data.center.0, data.center.1),
    );
    y = row(
        canvas,
        left,
        right,
        y,
        "VIEW",
        &format!("{}X{}", data.view.0, data.view.1),
    );
    y = row(
        canvas,
        left,
        right,
        y,
        "PIXEL",
        &format!("{}PX", data.pixels_per_cell),
    );

    y = separator(canvas, left, right, y);

    // Производительность.
    y = heading(canvas, left, y, "PERF");
    y = row(canvas, left, right, y, "FPS", &format!("{:.0}", data.fps));
    y = row(
        canvas,
        left,
        right,
        y,
        "FRAME",
        &format!("{:.1}MS", data.frame_time),
    );

    y = separator(canvas, left, right, y);

    // Задел под симуляцию.
    y = heading(canvas, left, y, "SIM");
    y = row(canvas, left, right, y, "TICK", &format!("{}", data.tick));
    y = row(canvas, left, right, y, "CELLS", &format!("{}", data.cells_alive));

    y = separator(canvas, left, right, y);

    // Управление.
    y = row(canvas, left, right, y, "WASD", "MOVE");
    y = row(canvas, left, right, y, "WHEEL", "ZOOM");
    row(canvas, left, right, y, "+/-", "ZOOM");
}

/// Название раздела.
fn heading(canvas: &mut Canvas, x: i32, y: i32, title: &str) -> i32 {
    draw_text(canvas, x, y, title, HEADING, TEXT_SCALE);

    y + LINE_HEIGHT
}

/// Строка «метка ... значение»: значение прижато к правому краю.
fn row(canvas: &mut Canvas, left: i32, right: i32, y: i32, label: &str, value: &str) -> i32 {
    draw_text(canvas, left, y, label, TEXT, TEXT_SCALE);

    let width = text_width(value, TEXT_SCALE) as i32;
    draw_text(canvas, right - width, y, value, TEXT, TEXT_SCALE);

    y + LINE_HEIGHT
}

/// Разделитель секций.
fn separator(canvas: &mut Canvas, left: i32, right: i32, y: i32) -> i32 {
    canvas.fill_rect(left, y, (right - left) as u32, 1, SEPARATOR);

    y + 1 + SECTION_GAP
}

/// Значение поля: число или прочерк, если клетки нет.
fn value(value: Option<f32>) -> String {
    match value {
        Some(value) => format_value(value),
        None => "-".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Колонка рисуется справа.
    #[test]
    fn menu_is_drawn_on_the_right() {
        let mut canvas = Canvas::new(1024, 1024);
        let data = MenuData::default();

        draw(&mut canvas, &data);

        let pixels = canvas.pixels();
        let side = 1024;

        // У правого края окна — фон колонки.
        assert_eq!(pixels[100 * side + side - 1], BACKGROUND);
    }

    /// Колонка не залезает на пространство.
    #[test]
    fn menu_does_not_cover_the_viewport() {
        let mut canvas = Canvas::new(1024, 1024);
        let data = MenuData::default();

        draw(&mut canvas, &data);

        let pixels = canvas.pixels();
        let side = 1024;
        let viewport_right = side - WIDTH as usize;

        // Пиксель сразу слева от колонки — не её фон.
        assert_ne!(pixels[100 * side + viewport_right - 1], BACKGROUND);
    }
}
