//! Окно с пространством: создание окна, цикл событий и вывод кадра.

use std::collections::HashSet;
use std::num::NonZeroU32;
use std::rc::Rc;
use std::time::{Duration, Instant};

use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop, OwnedDisplayHandle};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

use crate::camera::{Camera, MOVE_SPEED, ZOOM_STEP};
use crate::canvas::Canvas;
use crate::cell_colors::CellColors;
use crate::panel;
use crate::render::{WINDOW_SIZE, render};
use crate::space::Space;
use crate::space_gradients;

/// Заголовок окна.
const WINDOW_TITLE: &str = "creatures-rs";

/// Длительность кадра анимации (60 кадров в секунду).
const FRAME_TIME: Duration = Duration::from_millis(16);

/// Контекст `softbuffer`, привязанный к дисплею, на котором запущено приложение.
type BufferContext = softbuffer::Context<OwnedDisplayHandle>;

/// Поверхность `softbuffer`, в которую рисуем содержимое окна.
type BufferSurface = softbuffer::Surface<OwnedDisplayHandle, Rc<Window>>;

/// Окно вместе с поверхностью для рисования.
struct WindowState {
    /// Окно (в `Rc`, чтобы поверхность могла держать на него ссылку).
    window: Rc<Window>,
    /// Поверхность, в которую рисуются пиксели окна.
    surface: BufferSurface,
}

/// Приложение: контекст отрисовки, данные пространства и созданное окно.
struct SpaceApp {
    /// Контекст `softbuffer`, нужен для создания поверхности.
    context: BufferContext,
    /// Пространство, которое рисуется в окно.
    space: Space,
    /// Камера: масштаб и центр обзора.
    camera: Camera,
    /// Позиция курсора в окне (в физических пикселях), если он внутри.
    cursor: Option<(f64, f64)>,
    /// Камера перетаскивается мышью.
    dragging: bool,
    /// Удерживаемые клавиши управления камерой.
    pressed: HashSet<KeyCode>,
    /// Момент последнего кадра — нужен для скорости движения.
    last_frame: Instant,
    /// Когда наступает следующий кадр анимации.
    next_frame: Instant,
    /// Холст кадра: переиспользуется между кадрами, чтобы не выделять память.
    frame: Canvas,
    /// Кэш цветов клеток — пересчитывается, только когда меняется пространство.
    colors: CellColors,
    /// Пространство изменилось: цвета пора пересчитать.
    space_changed: bool,
    /// Окно появляется только после события `Resumed`.
    state: Option<WindowState>,
}

impl SpaceApp {
    /// Просит перерисовать окно, если оно уже создано.
    fn request_redraw(&self) {
        if let Some(state) = &self.state {
            state.window.request_redraw();
        }
    }

    /// Просит цикл событий проснуться через кадр (для анимации).
    fn keep_animating(&self, event_loop: &ActiveEventLoop) {
        event_loop.set_control_flow(ControlFlow::WaitUntil(Instant::now() + FRAME_TIME));
    }

    /// Размер окна в физических пикселях.
    fn window_size(&self) -> (u32, u32) {
        match &self.state {
            Some(state) => {
                let size = state.window.inner_size();

                (size.width, size.height)
            }
            None => (0, 0),
        }
    }

    /// Место для будущей симуляции: правит пространство на месте.
    ///
    /// Сейчас пространство только заполняется градиентами при старте,
    /// поэтому шаг ничего не делает. Когда появится эволюция клеток,
    /// изменения сеток (`grid_o2`, `grid_light`, `grid_minerals`) делаются
    /// здесь, а окно отрисует их на следующем кадре.
    fn update(&mut self) {
        // После изменения пространства цвета клеток надо пересчитать:
        // self.space_changed = true;
    }

    /// Собирает кадр: пространство и инфо-панель поверх него.
    fn frame(&mut self, width: u32, height: u32) {
        if self.space_changed {
            self.colors.rebuild(&self.space);
            self.space_changed = false;
        }

        self.frame.resize(width, height);

        render(&self.camera, &self.colors, &mut self.frame);

        // Инфо-панель рисуется только если курсор в окне.
        if let Some(cursor) = self.cursor {
            panel::draw(&mut self.frame, &self.space, &self.camera, cursor);
        }
    }

    /// Рисует пространство и выводит кадр в окно.
    fn draw(&mut self) {
        self.update();

        let size = {
            let Some(state) = &self.state else {
                return;
            };

            state.window.inner_size()
        };

        let (Some(width), Some(height)) =
            (NonZeroU32::new(size.width), NonZeroU32::new(size.height))
        else {
            // Окно свёрнуто или имеет нулевой размер — рисовать нечего.
            return;
        };

        self.frame(width.get(), height.get());

        let Some(state) = &mut self.state else {
            return;
        };

        if state.surface.resize(width, height).is_err() {
            return;
        }

        let Ok(mut buffer) = state.surface.buffer_mut() else {
            return;
        };

        let frame = self.frame.pixels();
        let len = frame.len().min(buffer.len());
        buffer[..len].copy_from_slice(&frame[..len]);

        let _ = buffer.present();

        // Следующий кадр запрашиваем сразу после вывода: на Wayland именно так
        // событие перерисовки доходит до приложения.
        self.request_redraw();
    }

    /// Приближает или отдаляет камеру вокруг точки курсора.
    fn zoom(&mut self, factor: f32) {
        let (width, height) = self.window_size();

        if width == 0 || height == 0 {
            return;
        }

        self.camera.zoom_at(factor, self.cursor, width, height);
        self.request_redraw();
    }

    /// Плавно сдвигает камеру по нажатым клавишам.
    ///
    /// Скорость постоянная в пикселях экрана: издалека карта пролетает за
    /// пару секунд, а вблизи камера ползёт по клеткам.
    fn move_camera(&mut self, dt: f32) -> bool {
        let mut axis_x = 0.0;
        let mut axis_y = 0.0;

        if self.pressed.contains(&KeyCode::KeyA) {
            axis_x -= 1.0;
        }
        if self.pressed.contains(&KeyCode::KeyD) {
            axis_x += 1.0;
        }
        if self.pressed.contains(&KeyCode::KeyW) {
            axis_y -= 1.0;
        }
        if self.pressed.contains(&KeyCode::KeyS) {
            axis_y += 1.0;
        }

        if axis_x == 0.0 && axis_y == 0.0 {
            return false;
        }

        let distance = MOVE_SPEED * dt;

        self.camera.pan(axis_x * distance, axis_y * distance);

        true
    }
}

impl ApplicationHandler for SpaceApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.state.is_some() {
            // Окно уже создано (повторный `Resumed` бывает на мобильных).
            return;
        }

        let attributes = Window::default_attributes()
            .with_title(WINDOW_TITLE)
            .with_inner_size(PhysicalSize::new(WINDOW_SIZE, WINDOW_SIZE));

        let window = match event_loop.create_window(attributes) {
            Ok(window) => Rc::new(window),
            Err(err) => {
                eprintln!("Не удалось создать окно: {err}");
                event_loop.exit();
                return;
            }
        };

        let surface = match BufferSurface::new(&self.context, window.clone()) {
            Ok(surface) => surface,
            Err(err) => {
                eprintln!("Не удалось создать поверхность для рисования: {err}");
                event_loop.exit();
                return;
            }
        };

        window.request_redraw();
        self.state = Some(WindowState { window, surface });
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),

            WindowEvent::Resized(_) | WindowEvent::ScaleFactorChanged { .. } => {
                self.request_redraw();
            }

            // Колесо мыши приближает и отдаляет камеру.
            WindowEvent::MouseWheel { delta, .. } => {
                let factor = match delta {
                    MouseScrollDelta::LineDelta(_, lines) => ZOOM_STEP.powf(lines),
                    MouseScrollDelta::PixelDelta(pixels) => 1.0015f32.powf(pixels.y as f32),
                };

                self.zoom(factor);
                self.next_frame = Instant::now();
                self.keep_animating(event_loop);
            }

            // Зажатая левая кнопка мыши тянет камеру.
            WindowEvent::MouseInput { state, button, .. } => {
                if button == MouseButton::Left {
                    self.dragging = state == ElementState::Pressed;
                }
            }

            WindowEvent::CursorMoved { position, .. } => {
                let previous = self.cursor;
                self.cursor = Some((position.x, position.y));

                if self.dragging && let Some((x, y)) = previous {
                    // Содержимое едет за курсором, поэтому знак обратный.
                    self.camera
                        .pan(x as f32 - position.x as f32, y as f32 - position.y as f32);
                }
                self.request_redraw();
            }

            WindowEvent::CursorLeft { .. } => {
                self.cursor = None;
                self.dragging = false;
                self.request_redraw();
            }

            // WASD двигает камеру, плюс и минус меняют масштаб.
            WindowEvent::KeyboardInput { event, .. } => {
                let PhysicalKey::Code(code) = event.physical_key else {
                    return;
                };

                if !is_camera_key(code) {
                    return;
                }

                match event.state {
                    ElementState::Pressed => {
                        // Автоповтор ОС присылает нажатия снова — зум на них
                        // не должен срабатывать.
                        if self.pressed.insert(code) {
                            // Отсчёт времени — с момента нажатия, иначе первый
                            // кадр дёрнется на всё накопленное время простоя.
                            self.last_frame = Instant::now();
                            self.next_frame = Instant::now();

                            match code {
                                KeyCode::Equal | KeyCode::NumpadAdd => self.zoom(ZOOM_STEP),
                                KeyCode::Minus | KeyCode::NumpadSubtract => {
                                    self.zoom(1.0 / ZOOM_STEP)
                                }
                                _ => {}
                            }
                        }

                        self.keep_animating(event_loop);
                    }
                    ElementState::Released => {
                        self.pressed.remove(&code);
                    }
                }
            }

            WindowEvent::RedrawRequested => self.draw(),
            _ => {}
        }
    }

    /// Покадровое обновление: движение камеры и анимация зума.
    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let now = Instant::now();

        // Ничего не движется — спим до следующего события.
        if self.pressed.is_empty() && !self.camera.is_zooming() {
            event_loop.set_control_flow(ControlFlow::Wait);
            return;
        }

        // Темп кадров держим сами: если цикл событий проснулся раньше срока,
        // досыпаем до него. Так расписание не зависит от того, как бэкенд
        // winit ждёт событий (в X11 ожидание отменяется, если в очереди
        // есть запрос на перерисовку, и цикл начинает крутиться без остановки).
        if now < self.next_frame {
            std::thread::sleep(self.next_frame - now);
        }

        let now = Instant::now();
        let dt = now.duration_since(self.last_frame).as_secs_f32().min(0.05);
        self.last_frame = now;
        self.next_frame = now + FRAME_TIME;

        self.move_camera(dt);
        self.camera.update(dt);

        self.request_redraw();

        event_loop.set_control_flow(ControlFlow::WaitUntil(self.next_frame));
    }
}

/// Клавиши управления камерой.
fn is_camera_key(code: KeyCode) -> bool {
    matches!(
        code,
        KeyCode::KeyW
            | KeyCode::KeyA
            | KeyCode::KeyS
            | KeyCode::KeyD
            | KeyCode::Equal
            | KeyCode::Minus
            | KeyCode::NumpadAdd
            | KeyCode::NumpadSubtract
    )
}

/// Создаёт окно с пространством и запускает цикл событий.
///
/// Ошибки (например, отсутствие дисплея) печатаются в stderr, чтобы не путать
/// их с ошибками самого симулятора.
pub fn run() {
    if let Err(err) = run_inner() {
        eprintln!("Не удалось запустить окно: {err}");
    }
}

/// Внутренняя реализация [`run`], возвращающая ошибку вместо её печати.
fn run_inner() -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = EventLoop::new()?;
    let context = BufferContext::new(event_loop.owned_display_handle())?;

    let mut app = SpaceApp {
        context,
        space: space_gradients::gradients(),
        camera: Camera::new(),
        cursor: None,
        dragging: false,
        pressed: HashSet::new(),
        last_frame: Instant::now(),
        next_frame: Instant::now(),
        frame: Canvas::new(0, 0),
        colors: CellColors::new(),
        // Цвета считаются перед первым кадром.
        space_changed: true,
        state: None,
    };

    event_loop.set_control_flow(ControlFlow::Wait);
    event_loop.run_app(&mut app)?;

    Ok(())
}
