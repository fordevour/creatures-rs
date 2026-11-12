//! Цвет клетки пространства: белый фон с наложенными оттенками полей.

use crate::palette::{BACKGROUND, LIGHT_COLOR, MINERALS_COLOR, O2_COLOR};
use crate::space::Space;

/// Цвет клетки по её координатам.
///
/// Чем больше значение поля в клетке, тем сильнее цвет уходит в сторону
/// оттенка этого поля, а не в чёрный.
pub fn cell_color(space: &Space, x: usize, y: usize) -> u32 {
    let mut rgb = BACKGROUND;

    for (value, tint) in [
        (space.light_at(x, y), LIGHT_COLOR),
        (space.o2_at(x, y), O2_COLOR),
        (space.minerals_at(x, y), MINERALS_COLOR),
    ] {
        let strength = value.clamp(0.0, 1.0);

        for (channel, tint_channel) in rgb.iter_mut().zip(tint) {
            *channel *= (1.0 - strength) + strength * (tint_channel / 255.0);
        }
    }

    let red = rgb[0].round() as u32;
    let green = rgb[1].round() as u32;
    let blue = rgb[2].round() as u32;

    (red << 16) | (green << 8) | blue
}
