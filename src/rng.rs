//! Псевдослучайный генератор чисел.
//!
//! Алгоритм — xoshiro256++: быстрый, с хорошей статистикой и без «сетевых
//! артефактов» простых генераторов. Зерно раздаётся через SplitMix64.
//!
//! Генератор детерминирован: одинаковое зерно — одинаковая последовательность
//! чисел. Это нужно симуляции, чтобы опыт можно было повторить.
//!
//! ```
//! use creatures_rs::rng::Rng;
//!
//! let mut rng = Rng::new(42);
//!
//! assert_eq!(rng.next_u64(), Rng::new(42).next_u64());
//! ```

/// Генератор псевдослучайных чисел.
#[derive(Clone, Debug)]
pub struct Rng {
    /// Состояние генератора.
    state: [u64; 4],
}

impl Rng {
    /// Генератор с заданным зерном.
    ///
    /// Одинаковое зерно всегда даёт одинаковую последовательность.
    pub fn new(seed: u64) -> Self {
        // Нулевое состояние вырождается, поэтому заменяем его на заведомо
        // ненулевое.
        let seed = if seed == 0 { 0x9E37_79B9_7F4A_7C15 } else { seed };

        let mut mixer = seed;
        let mut state = [0u64; 4];

        for slot in state.iter_mut() {
            *slot = splitmix64(&mut mixer);
        }

        Rng { state }
    }

    /// Генератор со случайным зерном.
    ///
    /// Зерно берётся из времени, адреса в памяти и счётчика — чтобы два
    /// генератора, созданных подряд, не совпали.
    pub fn from_entropy() -> Self {
        use std::sync::atomic::{AtomicU64, Ordering};
        use std::time::{SystemTime, UNIX_EPOCH};

        static COUNTER: AtomicU64 = AtomicU64::new(0);

        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0);

        let counter = COUNTER.fetch_add(1, Ordering::Relaxed);
        let stack = &counter as *const u64 as u64;

        Rng::new(nanos ^ stack.rotate_left(17) ^ counter.wrapping_mul(0x2545_F491_4F6C_DD1D))
    }

    /// Следующее 64-битное число.
    pub fn next_u64(&mut self) -> u64 {
        let result = self.state[0]
            .wrapping_add(self.state[3])
            .rotate_left(23)
            .wrapping_add(self.state[0]);

        let t = self.state[1] << 17;

        self.state[2] ^= self.state[0];
        self.state[3] ^= self.state[1];
        self.state[1] ^= self.state[2];
        self.state[0] ^= self.state[3];
        self.state[2] ^= t;
        self.state[3] = self.state[3].rotate_left(45);

        result
    }

    /// Следующее 32-битное число.
    pub fn next_u32(&mut self) -> u32 {
        (self.next_u64() >> 32) as u32
    }

    /// Случайное число в диапазоне `[0, 1)`.
    pub fn next_f32(&mut self) -> f32 {
        // 24 бита — ровно столько нужно для float без потери точности.
        (self.next_u32() >> 8) as f32 / (1u32 << 24) as f32
    }

    /// Случайное число в диапазоне `[0, 1)`.
    pub fn next_f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }

    /// Случайное целое в диапазоне `[low, high)`.
    ///
    /// # Panics
    ///
    /// Паникует, если `high <= low`.
    pub fn range(&mut self, low: i64, high: i64) -> i64 {
        assert!(high > low, "диапазон должен быть непустым");

        let span = (high - low) as u64;

        // Умножение с расширением: без перекоса, как при взятии по модулю.
        let offset = ((self.next_u64() as u128 * span as u128) >> 64) as u64;

        low + offset as i64
    }

    /// Случайное число в диапазоне `[low, high)`.
    ///
    /// # Panics
    ///
    /// Паникует, если `high <= low`.
    pub fn range_f32(&mut self, low: f32, high: f32) -> f32 {
        assert!(high > low, "диапазон должен быть непустым");

        low + self.next_f32() * (high - low)
    }

    /// Случайное число с нормальным распределением (среднее 0, отклонение 1).
    ///
    /// Нужно для «естественных» значений: шум, разброс скоростей и так далее.
    pub fn gaussian(&mut self) -> f64 {
        // Метод Бокса–Мюллера: два нормальных числа из двух равномерных.
        let mut u1 = self.next_f64();
        let u2 = self.next_f64();

        if u1 <= f64::MIN_POSITIVE {
            u1 = f64::MIN_POSITIVE;
        }

        let radius = (-2.0 * u1.ln()).sqrt();
        let angle = std::f64::consts::TAU * u2;

        radius * angle.cos()
    }

    /// Случайное число с нормальным распределением, ограниченным диапазоном.
    pub fn gaussian_range(&mut self, mean: f32, deviation: f32) -> f32 {
        (mean + self.gaussian() as f32 * deviation).clamp(0.0, 1.0)
    }

    /// Вероятность `probability` (0.0 — никогда, 1.0 — всегда).
    pub fn chance(&mut self, probability: f32) -> bool {
        self.next_f32() < probability
    }

    /// Случайный элемент среза.
    pub fn pick<'a, T>(&mut self, items: &'a [T]) -> Option<&'a T> {
        if items.is_empty() {
            return None;
        }

        let index = self.range(0, items.len() as i64) as usize;

        items.get(index)
    }

    /// Перемешивает срез на месте.
    pub fn shuffle<T>(&mut self, items: &mut [T]) {
        for i in (1..items.len()).rev() {
            let j = self.range(0, i as i64 + 1) as usize;

            items.swap(i, j);
        }
    }
}

/// SplitMix64: расширяет одно зерно в четыре слова состояния.
fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);

    let mut z = *state;

    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);

    z ^ (z >> 31)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Одинаковое зерно — одинаковая последовательность.
    #[test]
    fn same_seed_gives_same_sequence() {
        let mut a = Rng::new(42);
        let mut b = Rng::new(42);

        for _ in 0..100 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }

    /// Разные зёрна — разные последовательности.
    #[test]
    fn different_seeds_differ() {
        let mut a = Rng::new(1);
        let mut b = Rng::new(2);

        let same = (0..100).all(|_| a.next_u64() == b.next_u64());

        assert!(!same);
    }

    /// Дробные числа лежат в [0, 1).
    #[test]
    fn fractions_are_in_range() {
        let mut rng = Rng::new(7);

        for _ in 0..10_000 {
            let value = rng.next_f32();

            assert!((0.0..1.0).contains(&value));

            let value = rng.next_f64();

            assert!((0.0..1.0).contains(&value));
        }
    }

    /// Целые числа попадают в заданный диапазон.
    #[test]
    fn range_is_respected() {
        let mut rng = Rng::new(3);

        for _ in 0..10_000 {
            let value = rng.range(-5, 5);

            assert!((-5..5).contains(&value));

            let value = rng.range(10, 20);

            assert!((10..20).contains(&value));
        }
    }

    /// Значения распределены по диапазону примерно равномерно.
    #[test]
    fn distribution_is_even() {
        let mut rng = Rng::new(11);

        let mut buckets = [0u32; 10];

        for _ in 0..100_000 {
            buckets[rng.range(0, 10) as usize] += 1;
        }

        // Каждая десятая должна набрать около 10 000: допускаем разброс в 10%.
        for count in buckets {
            assert!(
                (9_000..=11_000).contains(&count),
                "неравномерно: {buckets:?}"
            );
        }
    }

    /// Выбор элемента и перемешивание работают.
    #[test]
    fn pick_and_shuffle_work() {
        let mut rng = Rng::new(5);

        let items = [1, 2, 3, 4, 5];

        assert!(rng.pick(&items).is_some());
        assert!(rng.pick::<u8>(&[]).is_none());

        let mut items = [1, 2, 3, 4, 5];
        rng.shuffle(&mut items);

        let mut sorted = items;
        sorted.sort_unstable();

        assert_eq!(sorted, [1, 2, 3, 4, 5]);
    }

    /// Нормальное распределение в среднем около нуля.
    #[test]
    fn gaussian_is_centered() {
        let mut rng = Rng::new(13);

        let mut sum = 0.0;

        for _ in 0..100_000 {
            sum += rng.gaussian();
        }

        let mean = sum / 100_000.0;

        assert!(mean.abs() < 0.02, "среднее: {mean}");
    }
}
