//! `creatures-rs` — пространство с клетками и окно, в котором оно видно.
//!
//! Модули:
//! - [`constants`] — размеры мира и коэффициенты затухания полей;
//! - [`space`] — модель пространства с тремя сетками;
//! - [`space_gradients`] — начальное заполнение пространства;
//! - [`canvas`] — буфер пикселей и примитивы рисования;
//! - [`palette`] — цвета полей;
//! - [`cell_color`] — цвет клетки по значениям полей;
//! - [`cell_colors`] — кэш цветов всех клеток;
//! - [`render`] — отрисовка пространства в холст;
//! - [`font`] — самописный шрифт 5x7;
//! - [`text`] — вывод текста;
//! - [`value`] — форматирование значений полей;
//! - [`camera`] — камера: масштаб, центр обзора и клетка под курсором;
//! - [`panel`] — инфо-панель;
//! - [`window`] — окно и цикл событий.

pub mod camera;
pub mod canvas;
pub mod cell_color;
pub mod cell_colors;
pub mod constants;
pub mod font;
pub mod panel;
pub mod palette;
pub mod render;
pub mod space;
pub mod space_gradients;
pub mod text;
pub mod value;
pub mod window;
