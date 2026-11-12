//! Счётчик кадров: FPS и время последней отрисовки.
//!
//! FPS считается по интервалам между кадрами — это настоящая частота
//! отрисовки. Время кадра (сколько длилась отрисовка) показывается отдельно:
//! оно меняется от нагрузки, а FPS — от темпа.

use std::time::Duration;

/// Сколько последних кадров усредняем для FPS.
const WINDOW: usize = 30;

/// Интервал больше этого — простой, а не просадка скорости.
const IDLE_GAP: Duration = Duration::from_secs(1);

/// Счётчик производительности отрисовки.
pub struct Perf {
    /// Интервалы между кадрами.
    intervals: Vec<Duration>,
    /// Длительность последней отрисовки.
    last_draw: Duration,
}

impl Perf {
    /// Новый счётчик.
    pub fn new() -> Self {
        Perf {
            intervals: Vec::with_capacity(WINDOW),
            last_draw: Duration::ZERO,
        }
    }

    /// Учитывает очередной кадр.
    ///
    /// `interval` — сколько прошло с прошлого кадра, `draw` — сколько длилась
    /// отрисовка. Кадры после долгого простоя не учитываются: это не просадка.
    pub fn frame(&mut self, interval: Duration, draw: Duration) {
        self.last_draw = draw;

        if interval < IDLE_GAP {
            if self.intervals.len() == WINDOW {
                self.intervals.remove(0);
            }

            self.intervals.push(interval);
        }
    }

    /// Кадров в секунду по последним кадрам.
    pub fn fps(&self) -> f32 {
        if self.intervals.len() < 2 {
            return 0.0;
        }

        let total: Duration = self.intervals.iter().sum();

        if total.is_zero() {
            return 0.0;
        }

        self.intervals.len() as f32 / total.as_secs_f32()
    }

    /// Длительность последней отрисовки.
    pub fn last_draw(&self) -> Duration {
        self.last_draw
    }
}

impl Default for Perf {
    fn default() -> Self {
        Perf::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// FPS считается по интервалам между кадрами.
    #[test]
    fn fps_is_calculated_from_intervals() {
        let mut perf = Perf::new();

        for _ in 0..10 {
            perf.frame(Duration::from_millis(16), Duration::from_millis(2));
        }

        assert!((perf.fps() - 62.5).abs() < 1.0);
    }

    /// Время отрисовки не влияет на FPS.
    #[test]
    fn fps_does_not_depend_on_draw_time() {
        let mut perf = Perf::new();

        for _ in 0..10 {
            perf.frame(Duration::from_millis(16), Duration::from_millis(30));
        }

        assert!((perf.fps() - 62.5).abs() < 1.0);
        assert_eq!(perf.last_draw(), Duration::from_millis(30));
    }

    /// Простой не считается просадкой.
    #[test]
    fn idle_gap_is_ignored() {
        let mut perf = Perf::new();

        for _ in 0..10 {
            perf.frame(Duration::from_millis(16), Duration::from_millis(2));
        }

        // Кадр после долгого простоя.
        perf.frame(Duration::from_secs(5), Duration::from_millis(2));

        assert!((perf.fps() - 62.5).abs() < 1.0);
    }

    /// Один кадр — FPS неизвестен.
    #[test]
    fn fps_needs_a_few_frames() {
        let mut perf = Perf::new();

        assert_eq!(perf.fps(), 0.0);

        perf.frame(Duration::from_millis(16), Duration::from_millis(2));

        assert_eq!(perf.fps(), 0.0);
    }
}
