//! Мини-канвас: рисование примитивов в буфер пикселей формата `0x00RRGGBB`.

/// Пиксельный холст фиксированного размера.
pub struct Canvas {
    pixels: Vec<u32>,
    width: u32,
    height: u32,
}

impl Canvas {
    /// Создаёт холст нужного размера.
    pub fn new(width: u32, height: u32) -> Self {
        Canvas {
            pixels: vec![0; width as usize * height as usize],
            width,
            height,
        }
    }

    /// Подгоняет холст под новый размер, сохраняя содержимое.
    ///
    /// Холст переиспользуется между кадрами, чтобы не выделять мегабайты
    /// пикселей каждый раз.
    pub fn resize(&mut self, width: u32, height: u32) {
        if self.width == width && self.height == height {
            return;
        }

        let len = width as usize * height as usize;

        self.pixels.resize(len, 0);
        self.width = width;
        self.height = height;
    }

    /// Размер холста по ширине.
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Размер холста по высоте.
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Пиксели холста (в том же порядке, что ожидает `softbuffer`).
    pub fn pixels(&self) -> &[u32] {
        &self.pixels
    }

    /// Заливает весь холст цветом.
    pub fn fill(&mut self, color: u32) {
        self.pixels.fill(color);
    }

    /// Ставит пиксель; точки вне холста игнорируются.
    pub fn set_pixel(&mut self, x: i32, y: i32, color: u32) {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return;
        }

        let index = y as usize * self.width as usize + x as usize;
        self.pixels[index] = color;
    }

    /// Заливает прямоугольник цветом; части за пределами холста обрезаются.
    pub fn fill_rect(&mut self, x: i32, y: i32, width: u32, height: u32, color: u32) {
        let x_start = x.max(0) as u32;
        let y_start = y.max(0) as u32;
        let x_end = (x + width as i32).min(self.width as i32).max(0) as u32;
        let y_end = (y + height as i32).min(self.height as i32).max(0) as u32;

        if x_start >= x_end || y_start >= y_end {
            return;
        }

        for py in y_start..y_end {
            let row = py as usize * self.width as usize;
            let (from, to) = (row + x_start as usize, row + x_end as usize);

            self.pixels[from..to].fill(color);
        }
    }

    /// Рисует рамку прямоугольника толщиной в один пиксель.
    pub fn draw_rect(&mut self, x: i32, y: i32, width: u32, height: u32, color: u32) {
        if width == 0 || height == 0 {
            return;
        }

        self.fill_rect(x, y, width, 1, color);
        self.fill_rect(x, y + height as i32 - 1, width, 1, color);
        self.fill_rect(x, y, 1, height, color);
        self.fill_rect(x + width as i32 - 1, y, 1, height, color);
    }
}
