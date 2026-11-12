//! Зона обзора: часть окна, в которой рисуется пространство.
//!
//! Окно делится на две части: слева — пространство (его и описывает
//! `Viewport`), справа — панель меню.

/// Прямоугольник обзора в окне.
#[derive(Clone, Copy, Debug)]
pub struct Viewport {
    /// Левый край зоны.
    pub x: u32,
    /// Верхний край зоны.
    pub y: u32,
    /// Ширина зоны.
    pub width: u32,
    /// Высота зоны.
    pub height: u32,
}

impl Viewport {
    /// Новая зона обзора.
    pub fn new(x: u32, y: u32, width: u32, height: u32) -> Self {
        Viewport {
            x,
            y,
            width,
            height
        }
    }

    /// Правый край зоны (не входит в неё).
    pub fn right(&self) -> u32 {
        self.x + self.width
    }

    /// Нижний край зоны (не входит в неё).
    pub fn bottom(&self) -> u32 {
        self.y + self.height
    }

    /// Находится ли точка внутри зоны.
    pub fn contains(&self, x: f64, y: f64) -> bool {
        x >= self.x as f64
            && x < self.right() as f64
            && y >= self.y as f64
            && y < self.bottom() as f64
    }
}
