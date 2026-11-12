//! Данные для меню: всё, что показывает правая колонка.

/// Значения полей в клетке под курсором.
#[derive(Clone, Copy, Default)]
pub struct CellInfo {
    /// Клетка под курсором (`None`, если курсор не над пространством).
    pub cell: Option<(usize, usize)>,
    /// Кислород в клетке.
    pub o2: Option<f32>,
    /// Свет в клетке.
    pub light: Option<f32>,
    /// Минералы в клетке.
    pub minerals: Option<f32>,
}

/// Данные меню на текущий кадр.
#[derive(Clone, Copy, Default)]
pub struct MenuData {
    /// Клетка под курсором и значения полей в ней.
    pub cell: CellInfo,
    /// Текущий масштаб камеры.
    pub zoom: f32,
    /// Центр обзора в клетках пространства.
    pub center: (f32, f32),
    /// Сколько клеток видно по горизонтали и вертикали.
    pub view: (usize, usize),
    /// Сколько пикселей занимает клетка.
    pub pixels_per_cell: u32,
    /// Кадров в секунду.
    pub fps: f32,
    /// Длительность последнего кадра.
    pub frame_time: f32,
    /// Тик симуляции.
    pub tick: u64,
    /// Живых клеток.
    pub cells_alive: u64,
}
