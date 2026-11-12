use std::time::Instant;

use creatures_rs::camera::Camera;
use creatures_rs::canvas::Canvas;
use creatures_rs::cell_colors::CellColors;
use creatures_rs::render;
use creatures_rs::space_gradients;
use creatures_rs::viewport::Viewport;

fn measure(name: &str, camera: &Camera, colors: &CellColors, viewport: &Viewport) {
    let mut canvas = Canvas::new(viewport.width, viewport.height);

    for _ in 0..3 {
        render::render(camera, colors, &mut canvas, viewport);
    }

    let start = Instant::now();
    let frames = 30;

    for _ in 0..frames {
        render::render(camera, colors, &mut canvas, viewport);
    }

    let elapsed = start.elapsed();
    let (min_x, max_x, min_y, max_y) = camera.visible_cells(viewport);

    println!(
        "{name:24} {:6.2} мс  клеток {}x{}  zoom {:.1}",
        elapsed.as_secs_f64() * 1000.0 / frames as f64,
        max_x - min_x + 1,
        max_y - min_y + 1,
        camera.zoom()
    );
}

#[test]
fn bench_render() {
    let space = space_gradients::gradients();
    let mut colors = CellColors::new();
    colors.rebuild(&space);

    let viewport = Viewport::new(0, 0, 1024, 1024);
    measure("весь обзор", &Camera::new(), &colors, &viewport);

    let mut middle = Camera::new();
    middle.zoom_at(32.0, Some((512.0, 512.0)), &viewport);
    for _ in 0..200 {
        middle.update(1.0 / 60.0);
    }
    measure("приближение в центре", &middle, &colors, &viewport);
}
