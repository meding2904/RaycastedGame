#![forbid(unsafe_code)]
//! The window, the input and the clock. Everything else in the game is headless.

use std::num::NonZeroU32;
use std::sync::Arc;
use std::time::{Duration, Instant};

use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{DeviceEvent, ElementState, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

use wolf3d::assets::Assets;
use wolf3d::audio::Audio;
use wolf3d::game::{Game, TICK_DT};
use wolf3d::hud;
use wolf3d::input::Input;
use wolf3d::render::{Renderer, FRAME_HEIGHT, FRAME_WIDTH};

const WINDOW_WIDTH: u32 = 1280;
const WINDOW_HEIGHT: u32 = 800;

/// The most simulation steps a single frame is allowed to catch up on. A long stall (a dragged
const MAX_CATCH_UP_TICKS: u32 = 6;

struct State {
    window: Arc<Window>,
    surface: softbuffer::Surface<Arc<Window>, Arc<Window>>,
    assets: Assets,
    renderer: Renderer,
    game: Game,
    audio: Audio,
    input: Input,
    /// Mouse motion accumulated since the last tick, in pixels
    mouse_dx: f32,
    frame: Vec<u8>,
    /// Per-destination-pixel source columns and rows for the nearest neighbour scale
    columns: Vec<usize>,
    rows: Vec<usize>,
    floor_index: usize,
    last: Instant,
    accumulator: f32,
    next_frame: Instant,
}

impl State {
    fn new(window: Arc<Window>) -> Option<Self> {
        let context = softbuffer::Context::new(window.clone()).ok()?;
        let surface = softbuffer::Surface::new(&context, window.clone()).ok()?;

        let assets = Assets::generate();
        let renderer = Renderer::new(&assets);
        let mut game = Game::new();
        game.skip_intro();
        let audio = Audio::new(&assets.sounds);
        let mut state = Self {
            window,
            surface,
            renderer,
            game,
            audio,
            assets,
            input: Input::default(),
            mouse_dx: 0.0,
            frame: vec![0; FRAME_WIDTH * FRAME_HEIGHT * 4],
            columns: Vec::new(),
            rows: Vec::new(),
            floor_index: usize::MAX,
            last: Instant::now(),
            accumulator: 0.0,
            next_frame: Instant::now(),
        };
        state.audio.start_music();
        state.resize();
        state.enter_floor_if_changed();
        Some(state)
    }

    fn resize(&mut self) {
        let size = self.window.inner_size();
        let width = NonZeroU32::new(size.width).unwrap_or(NonZeroU32::MIN);
        let height = NonZeroU32::new(size.height).unwrap_or(NonZeroU32::MIN);
        let _ = self.surface.resize(width, height);
    }

    /// Rebuilds the engine world when the game moved to a different floor
    fn enter_floor_if_changed(&mut self) {
        if self.floor_index != self.game.floor_index {
            self.floor_index = self.game.floor_index;
            self.renderer.enter_floor(&self.game, &self.assets);
        }
    }

    /// Advances the simulation by a fixed number of ticks and plays what happened
    fn tick(&mut self, ticks: u32) {
        for _ in 0..ticks {
            self.input.mouse_dx = std::mem::take(&mut self.mouse_dx);
            self.game.tick(&self.input);
            self.input.clear_edges();

            for sfx in &self.game.sounds {
                self.audio.play(*sfx);
            }
        }

        self.enter_floor_if_changed();
    }

    fn draw(&mut self) {
        self.renderer.sync(&self.game, &self.assets);
        Renderer::clear(&mut self.frame);
        self.renderer.draw_view(&self.game, &mut self.frame);
        self.renderer.draw_weapon(&self.game, &mut self.frame);
        hud::draw(&mut self.frame, &self.assets, &self.renderer, &self.game);
        self.present();
    }

    /// Scales the 320x200 frame into the window, centred, with black bars
    fn present(&mut self) {
        let size = self.window.inner_size();
        let dst_width = size.width as usize;
        let dst_height = size.height as usize;
        if dst_width == 0 || dst_height == 0 {
            return;
        }

        let Ok(mut buffer) = self.surface.buffer_mut() else {
            return;
        };
        buffer.fill(0);

        let (view_width, view_height) = if dst_width * FRAME_HEIGHT >= dst_height * FRAME_WIDTH {
            (dst_height * FRAME_WIDTH / FRAME_HEIGHT, dst_height)
        } else {
            (dst_width, dst_width * FRAME_HEIGHT / FRAME_WIDTH)
        };
        if view_width == 0 || view_height == 0 {
            let _ = buffer.present();
            return;
        }

        let left = (dst_width - view_width) / 2;
        let top = (dst_height - view_height) / 2;

        if self.columns.len() != view_width {
            self.columns = (0..view_width)
                .map(|x| x * FRAME_WIDTH / view_width)
                .collect();
        }
        if self.rows.len() != view_height {
            self.rows = (0..view_height)
                .map(|y| (y * FRAME_HEIGHT / view_height) * FRAME_WIDTH)
                .collect();
        }

        for (y, row) in self.rows.iter().enumerate() {
            let start = (top + y) * dst_width + left;
            let target = &mut buffer[start..start + view_width];
            for (pixel, column) in target.iter_mut().zip(self.columns.iter()) {
                let i = (row + column) * 4;
                *pixel = ((self.frame[i] as u32) << 16)
                    | ((self.frame[i + 1] as u32) << 8)
                    | (self.frame[i + 2] as u32);
            }
        }

        let _ = buffer.present();
    }

    fn set_key(&mut self, code: KeyCode, down: bool) {
        match code {
            KeyCode::KeyW | KeyCode::ArrowUp => {
                self.input.forward = down;
                self.input.up = down;
            }
            KeyCode::KeyS | KeyCode::ArrowDown => {
                self.input.back = down;
                self.input.down = down;
            }
            KeyCode::KeyA => self.input.strafe_left = down,
            KeyCode::KeyD => self.input.strafe_right = down,
            KeyCode::ArrowLeft => self.input.turn_right = down,
            KeyCode::ArrowRight => self.input.turn_left = down,
            KeyCode::ShiftLeft | KeyCode::ShiftRight => self.input.run = down,
            KeyCode::ControlLeft | KeyCode::ControlRight => self.input.shoot = down,
            KeyCode::Tab => {}
            KeyCode::Escape => self.input.escape = down,
            KeyCode::Enter | KeyCode::NumpadEnter => self.input.enter = down,
            KeyCode::Space => {
                self.input.space = down;
                self.input.use_key = down;
            }
            KeyCode::KeyE => self.input.use_key = down,
            KeyCode::KeyQ => self.input.pause_quit = down,
            KeyCode::Digit1 => self.input.weapon = down.then_some(0),
            KeyCode::Digit2 => self.input.weapon = down.then_some(1),
            KeyCode::Digit3 => self.input.weapon = down.then_some(2),
            KeyCode::Digit4 => self.input.weapon = down.then_some(3),
            _ => {}
        }
    }
}

#[derive(Default)]
struct App {
    state: Option<State>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.state.is_some() {
            return;
        }

        let attributes = Window::default_attributes()
            .with_title("Wolfenstein 3D — Rust")
            .with_inner_size(LogicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT))
            .with_min_inner_size(LogicalSize::new(FRAME_WIDTH as u32, FRAME_HEIGHT as u32));

        let Ok(window) = event_loop.create_window(attributes) else {
            event_loop.exit();
            return;
        };
        let window = Arc::new(window);

        window.set_cursor_visible(true);

        let Some(state) = State::new(window) else {
            eprintln!("could not create the render surface");
            event_loop.exit();
            return;
        };

        println!(
            "audio: {} ({} effects, {:.1}s of music)",
            if state.audio.is_available() {
                "on"
            } else {
                "unavailable"
            },
            state.audio.effect_count(),
            state.audio.music_seconds()
        );
        println!("controls: WASD move, arrows look, shift run, space use, 1-4 weapons, esc pause, M mute");

        self.state = Some(state);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let Some(state) = self.state.as_mut() else {
            return;
        };

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(_) => state.resize(),
            WindowEvent::Focused(false) => {
                // Never leave a key stuck down because focus went elsewhere
                state.input.clear_all();
                state.mouse_dx = 0.0;
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if let PhysicalKey::Code(code) = event.physical_key {
                    if code == KeyCode::KeyM
                        && event.state == ElementState::Pressed
                        && !event.repeat
                    {
                        state.audio.toggle_mute();
                    } else if !event.repeat {
                        state.set_key(code, event.state == ElementState::Pressed);
                    } else if event.state == ElementState::Released {
                        state.set_key(code, false);
                    }
                }
            }
            WindowEvent::MouseInput {
                state: button_state,
                button: MouseButton::Left,
                ..
            } => {
                state.input.shoot = button_state == ElementState::Pressed;
            }
            WindowEvent::RedrawRequested => {
                let now = Instant::now();
                let elapsed = (now - state.last).as_secs_f32().min(0.5);
                state.last = now;
                state.accumulator += elapsed;

                let mut ticks = 0;
                while state.accumulator >= TICK_DT && ticks < MAX_CATCH_UP_TICKS {
                    state.accumulator -= TICK_DT;
                    ticks += 1;
                }
                if ticks > 0 {
                    state.tick(ticks);
                }
                state.input.mouse_dx = 0.0;
                state.draw();
            }
            _ => {}
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        event: DeviceEvent,
    ) {
        let Some(state) = self.state.as_mut() else {
            return;
        };
        if let DeviceEvent::MouseMotion { delta } = event {
            state.mouse_dx += delta.0 as f32;
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let Some(state) = self.state.as_ref() else {
            event_loop.exit();
            return;
        };

        let now = Instant::now();
        let next = if now >= state.next_frame {
            state.window.request_redraw();
            now + Duration::from_secs_f32(TICK_DT)
        } else {
            state.next_frame
        };
        if let Some(state) = self.state.as_mut() {
            state.next_frame = next;
        }
        event_loop.set_control_flow(ControlFlow::WaitUntil(next));
    }
}

fn main() -> Result<(), winit::error::EventLoopError> {
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Wait);
    let mut app = App::default();
    event_loop.run_app(&mut app)
}
