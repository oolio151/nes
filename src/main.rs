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

struct App {
    window: Option<Arc<Window>>,
    pixels: Option<Pixels<'static>>,
    emu: Emulator,
    last_frame: Instant,
    frame_count: u32,
    fps_timer: Instant,
    buttons: u8,
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
        }
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
            WindowEvent::RedrawRequested => {
                self.emu.set_controller1(self.buttons);
                self.emu.run_one_frame();

                if let Some(pixels) = &mut self.pixels {
                    let fb = self.emu.cpu.framebuffer();
                    let frame = pixels.frame_mut();
                    for (dst, &(r, g, b)) in frame.chunks_exact_mut(4).zip(fb.iter()) {
                        dst[0] = r;
                        dst[1] = g;
                        dst[2] = b;
                        dst[3] = 255;
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
                    KeyCode::KeyR if pressed => { self.emu.reset(); }
                    _ => {}
                }
            }
            _ => {}
        }
    }
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