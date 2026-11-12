//! Значение поля: перевод числа в текст для инфо-панели.

/// Форматирует значение поля.
///
/// Обычные значения — тремя знаками после запятой, а совсем малые —
/// в экспоненциальном виде, чтобы не превращать их в нули.
pub fn format_value(value: f32) -> String {
    let value = value.clamp(0.0, 1.0);

    if value > 0.0 && value < 0.001 {
        format!("{value:.1e}")
    } else {
        format!("{value:.3}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Малые значения показываются в экспоненциальном виде.
    #[test]
    fn small_values_use_exponent() {
        assert_eq!(format_value(0.5), "0.500");
        assert_eq!(format_value(0.0), "0.000");
        assert_eq!(format_value(0.00002), "2.0e-5");
    }
}
