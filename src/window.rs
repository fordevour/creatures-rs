//! Окно с пространством: создание окна, цикл событий и вывод кадра.

use std::num::NonZeroU32;
use std::rc::Rc;

use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop, OwnedDisplayHandle};
use winit::window::{Window, WindowId};

use crate::canvas::Canvas;
use crate::panel;
use crate::render::{WINDOW_SIZE, render};
use crate::space::Space;
use crate::space_gradients;

/// Заголовок окна.
const WINDOW_TITLE: &str = "creatures-rs";

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
    /// Позиция курсора в окне (в физических пикселях), если он внутри.
    cursor: Option<(f64, f64)>,
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

    /// Место для будущей симуляции: правит пространство на месте.
    ///
    /// Сейчас пространство только заполняется градиентами при старте,
    /// поэтому шаг ничего не делает. Когда появится эволюция клеток,
    /// изменения сеток (`grid_o2`, `grid_light`, `grid_minerals`) делаются
    /// здесь, а окно отрисует их на следующем кадре.
    fn update(&mut self) {}

    /// Собирает кадр: пространство и инфо-панель поверх него.
    fn frame(&self, width: u32, height: u32) -> Canvas {
        let mut canvas = render(&self.space, width, height);

        // Инфо-панель рисуется только если курсор в окне.
        if let Some(cursor) = self.cursor {
            panel::draw(&mut canvas, &self.space, cursor);
        }

        canvas
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

        let canvas = self.frame(width.get(), height.get());

        let Some(state) = &mut self.state else {
            return;
        };

        if state.surface.resize(width, height).is_err() {
            return;
        }

        let Ok(mut buffer) = state.surface.buffer_mut() else {
            return;
        };

        let frame = canvas.pixels();
        let len = frame.len().min(buffer.len());
        buffer[..len].copy_from_slice(&frame[..len]);

        let _ = buffer.present();
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
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor = Some((position.x, position.y));
                self.request_redraw();
            }
            WindowEvent::CursorLeft { .. } => {
                self.cursor = None;
                self.request_redraw();
            }
            WindowEvent::RedrawRequested => self.draw(),
            _ => {}
        }
    }
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
        cursor: None,
        state: None,
    };

    event_loop.run_app(&mut app)?;

    Ok(())
}
