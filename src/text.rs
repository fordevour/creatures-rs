//! Вывод текста на холст самописным шрифтом.

use crate::canvas::Canvas;
use crate::font::{GLYPH_ADVANCE, GLYPH_WIDTH, glyph};

/// Рисует текст левым верхним углом в точке `(x, y)`.
pub fn draw_text(canvas: &mut Canvas, x: i32, y: i32, text: &str, color: u32, scale: u32) {
    let mut cursor_x = x;

    for symbol in text.chars() {
        let rows = glyph(symbol);

        for (row_index, row) in rows.iter().enumerate() {
            for column in 0..GLYPH_WIDTH {
                if row & (1 << (GLYPH_WIDTH - 1 - column)) == 0 {
                    continue;
                }

                let px = cursor_x + column as i32 * scale as i32;
                let py = y + row_index as i32 * scale as i32;

                canvas.fill_rect(px, py, scale, scale, color);
            }
        }

        cursor_x += GLYPH_ADVANCE as i32 * scale as i32;
    }
}
