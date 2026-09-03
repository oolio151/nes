use font8x8::{UnicodeFonts, BASIC_FONTS};
use pixels::{Pixels, SurfaceTexture};
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};
use std::sync::Arc;
use std::time::{Duration, Instant};
use winit::event::{ElementState, KeyEvent};
use winit::keyboard::{KeyCode, PhysicalKey};
use std::io::{self, Write};

use oolio151_nes::emulator::Emulator;

const WIDTH: u32 = 256;
const HEIGHT: u32 = 240;
const FRAME_TIME: Duration = Duration::from_nanos(16_639_267);
const NOTIFICATION_TIME: Duration = Duration::from_secs(3);

struct Notification {
    text: String,
    expires_at: Instant,
}

struct App {
    window: Option<Arc<Window>>,
    pixels: Option<Pixels<'static>>,
    emu: Emulator,
    last_frame: Instant,
    frame_count: u32,
    fps_timer: Instant,
    buttons: u8,
    notification: Option<Notification>,
}

impl App {
    fn new(emu: Emulator) -> Self {
        Self { 
            window: None, 
            pixels: None, 
            emu, 
            last_frame: Instant::now(),
            frame_count: 0,
            fps_timer: Instant::now(),
            buttons: 0,
            notification: None,
        }
    }

    fn notify(&mut self, text: impl Into<String>) {
        self.notification = Some(Notification {
            text: text.into(),
            expires_at: Instant::now() + NOTIFICATION_TIME,
        });
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = event_loop
            .create_window(Window::default_attributes().with_title("NES"))
            .unwrap();
        let window = Arc::new(window);

        let size = window.inner_size();
        let surface_texture = SurfaceTexture::new(size.width, size.height, window.clone());
        let pixels = Pixels::new(WIDTH, HEIGHT, surface_texture).unwrap();

        self.window = Some(window);
        self.pixels = Some(pixels);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                if size.width > 0 && size.height > 0 {
                    if let Some(pixels) = &mut self.pixels {
                        if let Err(e) = pixels.resize_surface(size.width, size.height) {
                            eprintln!("pixels.resize_surface() failed: {e}");
                            event_loop.exit();
                        }
                    }
                }
            }
            WindowEvent::RedrawRequested => {
                self.emu.set_controller1(self.buttons);
                self.emu.run_one_frame();

                if self
                    .notification
                    .as_ref()
                    .is_some_and(|notification| Instant::now() >= notification.expires_at)
                {
                    self.notification = None;
                }

                if let Some(pixels) = &mut self.pixels {
                    let fb = self.emu.cpu.framebuffer();
                    let frame = pixels.frame_mut();
                    for (dst, &(r, g, b)) in frame.chunks_exact_mut(4).zip(fb.iter()) {
                        dst[0] = r;
                        dst[1] = g;
                        dst[2] = b;
                        dst[3] = 255;
                    }
                    if let Some(notification) = &self.notification {
                        draw_notification(frame, &notification.text);
                    }
                    if let Err(e) = pixels.render() {
                        eprintln!("pixels.render() failed: {e}");
                        event_loop.exit();
                        return;
                    }
                }


                let elapsed = self.last_frame.elapsed();
                if elapsed < FRAME_TIME {
                    std::thread::sleep(FRAME_TIME - elapsed);
                }
                self.last_frame = Instant::now();
                
                if let Some(window) = &self.window {
                    window.request_redraw();
                }

                // this is used for showing the fps in the title bar
                self.frame_count += 1;
                if self.fps_timer.elapsed().as_secs_f32() >= 1.0 {
                    let fps = self.frame_count as f32 / self.fps_timer.elapsed().as_secs_f32();
                    if let Some(window) = &self.window {
                        window.set_title(&format!("NES Emulator - {:.1} FPS", fps));
                    }
                    self.frame_count = 0;
                    self.fps_timer = Instant::now();
                }
            }

            WindowEvent::KeyboardInput {
                event: KeyEvent { physical_key: PhysicalKey::Code(code), state, .. },
                ..
            } => {
                let pressed = state == ElementState::Pressed;
                // make sure to implement custom controls once you get to that
                let bit: u8 = match code {
                    KeyCode::KeyZ => 0x01, // a
                    KeyCode::KeyX => 0x02, // b
                    KeyCode::KeyA => 0x04, // select
                    KeyCode::KeyS => 0x08, // start
                    KeyCode::ArrowUp => 0x10,
                    KeyCode::ArrowDown => 0x20,
                    KeyCode::ArrowLeft => 0x40,
                    KeyCode::ArrowRight => 0x80,
                    _ => 0,
                };
                if pressed { self.buttons |= bit; } else { self.buttons &= !bit; }

                match code {
                    KeyCode::KeyR if pressed => {
                        self.emu.reset();
                        self.notify("RESET");
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
}

fn draw_notification(frame: &mut [u8], text: &str) {
    const GLYPH_WIDTH: u32 = 8;
    const GLYPH_HEIGHT: u32 = 8;
    const PADDING: u32 = 3;
    const MARGIN: u32 = 4;

    let text = text.to_ascii_uppercase();
    let text_width = text.chars().count() as u32 * GLYPH_WIDTH;
    let box_width = text_width + PADDING * 2;
    let box_height = GLYPH_HEIGHT + PADDING * 2;
    let box_x = WIDTH.saturating_sub(box_width + MARGIN);
    let box_y = HEIGHT.saturating_sub(box_height + MARGIN);

    fill_rect(frame, box_x, box_y, box_width, box_height, [0, 0, 0, 220]);

    for (character_index, character) in text.chars().enumerate() {
        let Some(glyph) = BASIC_FONTS.get(character) else {
            continue;
        };
        let glyph_x = box_x + PADDING + character_index as u32 * GLYPH_WIDTH;
        let glyph_y = box_y + PADDING;

        for (row, bits) in glyph.iter().enumerate() {
            for column in 0..GLYPH_WIDTH {
                if bits & (1 << column) != 0 {
                    put_pixel(frame, glyph_x + column, glyph_y + row as u32, [255, 255, 255, 255]);
                }
            }
        }
    }
}

fn fill_rect(frame: &mut [u8], x: u32, y: u32, width: u32, height: u32, color: [u8; 4]) {
    for py in y..(y + height).min(HEIGHT) {
        for px in x..(x + width).min(WIDTH) {
            put_pixel(frame, px, py, color);
        }
    }
}

fn put_pixel(frame: &mut [u8], x: u32, y: u32, color: [u8; 4]) {
    let index = ((y * WIDTH + x) * 4) as usize;
    frame[index..index + 4].copy_from_slice(&color);
}

fn main() {
    let rom_path = prompt_for_rom_path();
    let emu = Emulator::from_file(&rom_path).expect("failed to load ROM");

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::new(emu);
    event_loop.run_app(&mut app).unwrap();
}

fn prompt_for_rom_path() -> String {
    loop {
        print!("Enter path to ROM file: ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("failed to read input");

        let path = input.trim();

        if path.is_empty() {
            println!("Path cannot be empty, try again.");
            continue;
        }

        if !std::path::Path::new(path).exists() {
            println!("No file found at '{}', try again.", path);
            continue;
        }

        return path.to_string();
    }
}
