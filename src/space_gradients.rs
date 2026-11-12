//! Начальное заполнение пространства: градиенты полей от краёв.
//!
//! Градиенты строятся по правилам, описанным в [`crate::space::Space`]:
//! - кислород уменьшается сверху вниз;
//! - свет уменьшается слева направо (и немного сверху вниз);
//! - минералы уменьшаются снизу вверх;
//! - концентрация падает по геометрической прогрессии: на первой клетке от
//!   края 1.0, на второй коэффициент затухания, на третий — его квадрат.
//!
//! Коэффициент затухания у каждого поля свой: [`crate::constants::O2_DECAY`],
//! [`crate::constants::LIGHT_DECAY`], [`crate::constants::MINERALS_DECAY`].

use crate::constants::{LIGHT_DECAY, MINERALS_DECAY, O2_DECAY, SPACE_SIZE};
use crate::space::Space;

/// Создаёт пространство, заполненное градиентами.
pub fn gradients() -> Space {
    let mut space = Space::empty();

    // Затухание от края: 1.0, decay, decay^2, ...
    let o2_falloff = falloff_table(O2_DECAY);
    let light_falloff = falloff_table(LIGHT_DECAY);
    let minerals_falloff = falloff_table(MINERALS_DECAY);

    for y in 0..SPACE_SIZE {
        for (x, light_falloff) in light_falloff.iter().enumerate() {
            // Положение клетки по вертикали: 0..1.
            let fy = y as f32 / (SPACE_SIZE - 1) as f32;

            // Кислород: максимум сверху, к низу концентрация падает.
            let o2 = o2_falloff[y];
            // Свет: максимум слева (на всём левом краю ровно 1.0), к низу
            // чуть меньше. Дефицит от единицы растёт с расстоянием от края
            // и с глубиной, так что на самом краю света всегда полное.
            let light = (1.0 - (1.0 - light_falloff) * (1.0 + 0.25 * fy)).max(0.0);
            // Минералы: максимум снизу, к верху концентрация падает.
            let minerals = minerals_falloff[SPACE_SIZE - 1 - y];

            let i = Space::index(x, y);
            space.grid_o2[i] = o2;
            space.grid_light[i] = light;
            space.grid_minerals[i] = minerals;
        }
    }

    space
}

/// Таблица затухания: `1.0`, `decay`, `decay^2`, ... на всю длину пространства.
fn falloff_table(decay: f32) -> Vec<f32> {
    (0..SPACE_SIZE).map(|d| decay.powi(d as i32)).collect()
}
