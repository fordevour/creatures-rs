//! Камера: масштаб и центр обзора пространства.

use crate::constants::SPACE_SIZE;
use crate::render::PIXELS_PER_CELL;
use crate::viewport::Viewport;

/// Максимальный масштаб (во сколько раз клетка больше базовой).
pub const MAX_ZOOM: f32 = 32.0;

/// Во сколько раз меняется масштаб за одну «ступеньку» (колесо мыши).
pub const ZOOM_STEP: f32 = 1.15;

/// Скорость движения камеры в пикселях экрана в секунду.
///
/// Скорость постоянная в пикселях, поэтому вблизи камера ползёт по клеткам,
/// а издалека пролетает всю карту за пару секунд.
pub const MOVE_SPEED: f32 = 600.0;

/// Насколько быстро текущий масштаб догоняет целевой (в секунду).
pub const ZOOM_SMOOTHING: f32 = 12.0;

/// Самый мелкий масштаб: клетка не может стать нулевой.
const MIN_ZOOM: f32 = 0.01;

/// Точка пространства, которую надо держать под курсором во время зума.
struct Anchor {
    /// Мировая точка в клетках пространства.
    world: (f32, f32),
    /// Смещение этой точки относительно центра окна, в пикселях.
    offset: (f32, f32),
}

/// Камера над пространством.
pub struct Camera {
    /// Текущий масштаб: 1.0 — клетка занимает `PIXELS_PER_CELL` пикселей.
    zoom: f32,
    /// Масштаб, к которому текущий плавно стремится.
    target_zoom: f32,
    /// Центр обзора в клетках пространства.
    center: (f32, f32),
    /// Точка под курсором, которую нужно удерживать при зуме.
    anchor: Option<Anchor>,
}

impl Default for Camera {
    fn default() -> Self {
        Camera::new()
    }
}

impl Camera {
    /// Камера, показывающая всё пространство целиком.
    pub fn new() -> Self {
        let side = SPACE_SIZE as f32;

        Camera {
            zoom: 1.0,
            target_zoom: 1.0,
            center: (side / 2.0, side / 2.0),
            anchor: None,
        }
    }

    /// Текущий масштаб.
    pub fn zoom(&self) -> f32 {
        self.zoom
    }

    /// Целевой масштаб, к которому стремится камера.
    pub fn target_zoom(&self) -> f32 {
        self.target_zoom
    }

    /// Масштаб ещё догоняет цель (идёт анимация).
    pub fn is_zooming(&self) -> bool {
        (self.target_zoom - self.zoom).abs() > 1e-4
    }

    /// Пикселей в одной клетке при текущем масштабе.
    pub fn pixels_per_cell(&self) -> f32 {
        PIXELS_PER_CELL as f32 * self.zoom
    }

    /// Центр обзора в клетках пространства.
    pub fn center(&self) -> (f32, f32) {
        self.center
    }

    /// Минимальный масштаб: при нём всё пространство влезает в зону обзора.
    pub fn min_zoom(&self, viewport: &Viewport) -> f32 {
        let fit = (viewport.width.min(viewport.height)) as f32
            / (SPACE_SIZE as f32 * PIXELS_PER_CELL as f32);

        // Ниже нуля масштаб быть не может: клетка должна хоть что-то занимать.
        fit.clamp(MIN_ZOOM, 1.0)
    }

    /// Экранная координата левого верхнего угла клетки.
    ///
    /// `cx`/`cy` — координаты клетки в клетках пространства (дробные
    /// допустимы: так считаются границы).
    pub fn cell_origin(&self, cx: f32, cy: f32, viewport: &Viewport) -> (i32, i32) {
        let ppc = self.pixels_per_cell();

        let x = (cx - self.center.0) * ppc + viewport.x as f32 + viewport.width as f32 / 2.0;
        let y = (cy - self.center.1) * ppc + viewport.y as f32 + viewport.height as f32 / 2.0;

        (x.round() as i32, y.round() as i32)
    }

    /// Клетка под точкой экрана.
    pub fn cell_at(&self, cursor: (f64, f64), viewport: &Viewport) -> (usize, usize) {
        let ppc = self.pixels_per_cell();

        let x = (cursor.0 as f32 - viewport.x as f32 - viewport.width as f32 / 2.0) / ppc
            + self.center.0;
        let y = (cursor.1 as f32 - viewport.y as f32 - viewport.height as f32 / 2.0) / ppc
            + self.center.1;

        let x = (x.floor().max(0.0) as usize).min(SPACE_SIZE - 1);
        let y = (y.floor().max(0.0) as usize).min(SPACE_SIZE - 1);

        (x, y)
    }

    /// Диапазон видимых клеток (включительно).
    pub fn visible_cells(&self, viewport: &Viewport) -> (usize, usize, usize, usize) {
        let (min_x, min_y) = self.cell_at((viewport.x as f64, viewport.y as f64), viewport);
        let (max_x, max_y) = self.cell_at(
            (viewport.right() as f64 - 1.0, viewport.bottom() as f64 - 1.0),
            viewport,
        );

        (min_x, max_x, min_y, max_y)
    }

    /// Задаёт новый целевой масштаб, удерживая точку под курсором на месте.
    ///
    /// Сам масштаб меняется плавно — см. [`Camera::update`].
    pub fn zoom_at(&mut self, factor: f32, cursor: Option<(f64, f64)>, viewport: &Viewport) {
        let target = (self.target_zoom * factor).clamp(self.min_zoom(viewport), MAX_ZOOM);

        if target == self.target_zoom {
            return;
        }

        self.target_zoom = target;

        // Якорь — это точка пространства под курсором при текущем масштабе.
        // Её придётся удерживать на месте, пока масштаб догоняет цель.
        self.anchor = match cursor {
            Some((x, y)) => {
                let ppc = self.pixels_per_cell();

                let offset = (
                    x as f32 - viewport.x as f32 - viewport.width as f32 / 2.0,
                    y as f32 - viewport.y as f32 - viewport.height as f32 / 2.0,
                );

                Some(Anchor {
                    world: (
                        offset.0 / ppc + self.center.0,
                        offset.1 / ppc + self.center.1,
                    ),
                    offset,
                })
            }
            None => None,
        };
    }

    /// Плавно двигает масштаб к целевому. Возвращает `true`, если он изменился.
    pub fn update(&mut self, dt: f32) -> bool {
        let mut changed = false;

        if self.is_zooming() {
            let step = (ZOOM_SMOOTHING * dt).min(1.0);
            self.zoom += (self.target_zoom - self.zoom) * step;

            if !self.is_zooming() {
                self.zoom = self.target_zoom;
            }

            changed = true;
        }

        // Пока масштаб меняется, якорь держится под курсором.
        if let Some(anchor) = &self.anchor {
            let ppc = self.pixels_per_cell();

            self.center.0 = anchor.world.0 - anchor.offset.0 / ppc;
            self.center.1 = anchor.world.1 - anchor.offset.1 / ppc;

            self.clamp_center();

            if !self.is_zooming() {
                self.anchor = None;
            }
        }

        changed
    }

    /// Сдвигает камеру на клетки.
    pub fn pan_cells(&mut self, dx: f32, dy: f32) {
        self.center.0 += dx;
        self.center.1 += dy;

        self.anchor = None;
        self.clamp_center();
    }

    /// Сдвигает вид на пиксели экрана: положительный `dx` — вправо, `dy` — вниз.
    pub fn pan(&mut self, dx: f32, dy: f32) {
        let ppc = self.pixels_per_cell();

        self.center.0 += dx / ppc;
        self.center.1 += dy / ppc;

        self.anchor = None;
        self.clamp_center();
    }

    /// Не даёт увести камеру за пределы пространства.
    fn clamp_center(&mut self) {
        let side = SPACE_SIZE as f32;

        self.center.0 = self.center.0.clamp(0.0, side);
        self.center.1 = self.center.1.clamp(0.0, side);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Зона обзора во весь кадр.
    fn viewport() -> Viewport {
        Viewport::new(0, 0, 1024, 1024)
    }

    /// Точка экрана переводится в клетку пространства.
    #[test]
    fn cursor_maps_to_cell() {
        let camera = Camera::new();
        let viewport = viewport();

        assert_eq!(camera.cell_at((0.0, 0.0), &viewport), (0, 0));
        assert_eq!(camera.cell_at((100.0, 200.0), &viewport), (25, 50));
    }

    /// При увеличении клетка под курсором остаётся на месте.
    #[test]
    fn zoom_keeps_cell_under_cursor() {
        let mut camera = Camera::new();
        let cursor = (500.0, 300.0);

        let viewport = viewport();

        let before = camera.cell_at(cursor, &viewport);

        camera.zoom_at(ZOOM_STEP, Some(cursor), &viewport);
        camera.zoom_at(ZOOM_STEP, Some(cursor), &viewport);

        // Ждём конца анимации зума.
        for _ in 0..200 {
            camera.update(1.0 / 60.0);
        }

        assert_eq!(camera.cell_at(cursor, &viewport), before);
    }

    /// Масштаб не опускается ниже «видно всё пространство».
    #[test]
    fn zoom_is_limited() {
        let mut camera = Camera::new();

        let viewport = viewport();

        for _ in 0..100 {
            camera.zoom_at(1.0 / ZOOM_STEP, Some((0.0, 0.0)), &viewport);
        }

        assert!(camera.target_zoom() >= camera.min_zoom(&viewport));

        for _ in 0..200 {
            camera.zoom_at(ZOOM_STEP, Some((0.0, 0.0)), &viewport);
        }

        assert!(camera.target_zoom() <= MAX_ZOOM);
    }

    /// Масштаб меняется плавно, а не рывком.
    #[test]
    fn zoom_is_smooth() {
        let mut camera = Camera::new();

        camera.zoom_at(ZOOM_STEP, Some((512.0, 512.0)), &viewport());

        assert_ne!(camera.zoom(), camera.target_zoom());

        // За несколько кадров масштаб доходит до цели.
        for _ in 0..100 {
            camera.update(1.0 / 60.0);
        }

        assert!((camera.zoom() - camera.target_zoom()).abs() < 1e-3);
    }

    /// Камера едет с постоянной скоростью в пикселях экрана.
    #[test]
    fn move_speed_is_constant_in_pixels() {
        let mut camera = Camera::new();

        // Полсекунды движения — это 300 пикселей экрана.
        camera.pan(MOVE_SPEED * 0.5, 0.0);

        // В масштабе 1.0 одна клетка — 4 пикселя, значит сдвинулись 75 клеток.
        assert_eq!(camera.center().0, 128.0 + 75.0);
    }

    /// Направления движения: вверх — к верхнему краю пространства.
    #[test]
    fn pan_direction() {
        let mut camera = Camera::new();

        let center = camera.center();

        // Отрицательный `dy` — вверх по экрану, значит к началу пространства.
        camera.pan(0.0, -100.0);
        assert!(camera.center().1 < center.1);

        // Отрицательный `dx` — влево.
        camera.pan(-100.0, 0.0);
        assert!(camera.center().0 < center.0);
    }
}
