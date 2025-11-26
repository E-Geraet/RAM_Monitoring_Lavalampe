use pixels::{Pixels, SurfaceTexture};
use winit::{
    dpi::LogicalSize,
    event::{Event, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use image::{io::Reader as ImageReader, Rgba};
use sysinfo::{System, SystemExt};
use std::time::{Duration, Instant};
use std::path::PathBuf;
use std::collections::HashSet;
use std::sync::Mutex;
use std::env;

const WINDOW_SIZE: usize = 128;
const ANIMATION_FRAMES: usize = 169;

// --- ÄNDERUNG 1: XLarge hinzugefügt ---
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum WindowSizeMode {
    Small,
    Medium,
    Large,
    XLarge, 
}

impl WindowSizeMode {
    fn get_size(&self) -> usize {
        match self {
            WindowSizeMode::Small => WINDOW_SIZE,
            WindowSizeMode::Medium => WINDOW_SIZE * 2,
            WindowSizeMode::Large => WINDOW_SIZE * 4,
            // --- ÄNDERUNG 2: Hier steht jetzt korrekt XLarge ---
            WindowSizeMode::XLarge => WINDOW_SIZE * 8, 
        }
    }

    fn scale_up(&self) -> WindowSizeMode {
        match self {
            WindowSizeMode::Small => WindowSizeMode::Medium,
            WindowSizeMode::Medium => WindowSizeMode::Large,
            WindowSizeMode::Large => WindowSizeMode::XLarge,
            WindowSizeMode::XLarge => WindowSizeMode::XLarge,
        }
    }

    fn scale_down(&self) -> WindowSizeMode {
        match self {
            WindowSizeMode::Small => WindowSizeMode::Small,
            WindowSizeMode::Medium => WindowSizeMode::Small,
            WindowSizeMode::Large => WindowSizeMode::Medium,
            WindowSizeMode::XLarge => WindowSizeMode::Large,
        }
    }

    fn description(&self) -> &'static str {
        match self {
            WindowSizeMode::Small => "128x128",
            WindowSizeMode::Medium => "256x256",
            WindowSizeMode::Large => "512x512",
            WindowSizeMode::XLarge => "1024x1024",
        }
    }
}

lazy_static::lazy_static! {
    static ref ALREADY_PRINTED: Mutex<HashSet<String>> = Mutex::new(HashSet::new());
}

fn print_once(msg: &str) {
    let mut cache = ALREADY_PRINTED.lock().unwrap();
    if !cache.contains(msg) {
        println!("{}", msg);
        cache.insert(msg.to_string());
    }
}



fn find_asset_path(filename: &str) -> Option<PathBuf> {
    let path = PathBuf::from("assets").join(filename);
    if path.exists() { return Some(path); }
    
    if let Ok(exe_path) = env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let path = exe_dir.join("assets").join(filename);
            if path.exists() { return Some(path); }
            
            if let Some(parent_dir) = exe_dir.parent() {
                let path = parent_dir.join("assets").join(filename);
                if path.exists() { return Some(path); }
                
                if let Some(grandparent_dir) = parent_dir.parent() {
                    let path = grandparent_dir.join("assets").join(filename);
                    if path.exists() { return Some(path); }
                }
            }
        }
    }
    
    if let Some(home_dir) = env::var_os("HOME") {
        let path = PathBuf::from(home_dir)
            .join(".local/share/ram-lavalampe/assets")
            .join(filename);
        if path.exists() { return Some(path); }
    }
    
    None
}

fn load_lava_animation(filename: &str) -> Option<(Vec<Rgba<u8>>, usize, usize)> {
    let file_path = match find_asset_path(filename) {
        Some(path) => path,
        None => {
            eprintln!(">>> ERROR: Could not find asset file: {}", filename);
            return None;
        }
    };
    
    println!(">>> Attempting to load: {}", file_path.display());
    
    let img = match ImageReader::open(&file_path) {
        Ok(reader) => match reader.decode() {
            Ok(image) => image.to_rgba8(),
            Err(e) => {
                eprintln!("    ERROR: Failed to decode {}: {}", file_path.display(), e);
                return None;
            }
        },
        Err(e) => {
            eprintln!("    ERROR: Can't open {}: {}", file_path.display(), e);
            return None;
        }
    };

    let (width, height) = img.dimensions();
    let width = width as usize;
    let height = height as usize;

    if height != WINDOW_SIZE {
        eprintln!("    ERROR: Wrong height! Got {}, expected {}", height, WINDOW_SIZE);
        return None;
    }

    let expected_width = ANIMATION_FRAMES * WINDOW_SIZE;
    if width != expected_width {
        if width % WINDOW_SIZE != 0 {
            eprintln!("    ERROR: Width {} is not divisible by frame size {}", width, WINDOW_SIZE);
            return None;
        }
    }

    let mut pixel_data = Vec::with_capacity(width * height);
    for pixel in img.pixels() {
        pixel_data.push(*pixel);
    }

    Some((pixel_data, width, height))
}

fn blend_alpha(background: [u8; 4], foreground: Rgba<u8>) -> [u8; 4] {
    let [bg_r, bg_g, bg_b, bg_a] = background;
    let fg_r = foreground[0];
    let fg_g = foreground[1];
    let fg_b = foreground[2];
    let fg_a = foreground[3];

    if fg_a == 0 { return background; }
    if fg_a == 255 { return [fg_r, fg_g, fg_b, 255]; }

    let alpha = fg_a as f32 / 255.0;
    let inv_alpha = 1.0 - alpha;

    let r = (fg_r as f32 * alpha + bg_r as f32 * inv_alpha) as u8;
    let g = (fg_g as f32 * alpha + bg_g as f32 * inv_alpha) as u8;
    let b = (fg_b as f32 * alpha + bg_b as f32 * inv_alpha) as u8;
    let a = ((fg_a as f32 * alpha + bg_a as f32 * inv_alpha).min(255.0)) as u8;

    [r, g, b, a]
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting RAM Lava Lamp...");
    println!("Expected frame count: {}", ANIMATION_FRAMES);
    println!("Controls: Ctrl + Up Arrow = Scale Up, Ctrl + Down Arrow = Scale Down, Esc = Exit");

    let mut system = System::new_all();

    let event_loop = EventLoop::new();
    let window = {
        let size = LogicalSize::new(WINDOW_SIZE as f64, WINDOW_SIZE as f64);
        WindowBuilder::new()
            .with_title("RAM Lava Lamp")
            .with_inner_size(size)
            .with_min_inner_size(LogicalSize::new(WINDOW_SIZE as f64, WINDOW_SIZE as f64))
            // --- ÄNDERUNG 3: Max Size entfernt und Decorations auf true ---
            // .with_max_inner_size wurde entfernt!
            .with_resizable(true)
            .with_decorations(true) // Setze dies auf true, damit der Window Manager besser mitarbeitet
            .build(&event_loop)?
    };

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(WINDOW_SIZE as u32, WINDOW_SIZE as u32, surface_texture)?
    };

    let mut current_size_mode = WindowSizeMode::Small;
    let mut ctrl_pressed = false;

    let mut current_animation: Option<(Vec<Rgba<u8>>, usize, usize)> = None;
    let mut current_sprite_file = "";
    let mut frame_index = 0;
    let mut last_update = Instant::now();
    let mut last_ram_check = Instant::now();
    let mut current_ram_percent = 0.0;

    system.refresh_memory();
    print_once("RAM monitoring started");

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                println!("Shutting down...");
                *control_flow = ControlFlow::Exit;
            }
            Event::WindowEvent {
                event: WindowEvent::KeyboardInput { input, .. },
                ..
            } => {
                if let Some(keycode) = input.virtual_keycode {
                    match input.state {
                        winit::event::ElementState::Pressed => {
                            match keycode {
                                VirtualKeyCode::LControl | VirtualKeyCode::RControl => {
                                    ctrl_pressed = true;
                                }
                                VirtualKeyCode::Escape => {
                                    println!("Shutting down...");
                                    *control_flow = ControlFlow::Exit;
                                }
                                VirtualKeyCode::Up => {
                                    if ctrl_pressed {
                                        let old_size_mode = current_size_mode;
                                        current_size_mode = current_size_mode.scale_up();
                                        if current_size_mode != old_size_mode {
                                            let size = current_size_mode.get_size();
                                            println!("Scaling window up to {}", current_size_mode.description());
                                            let new_size = LogicalSize::new(size as f64, size as f64);
                                            window.set_inner_size(new_size);
                                            window.request_redraw();
                                        }
                                    }
                                }
                                VirtualKeyCode::Down => {
                                    if ctrl_pressed {
                                        let old_size_mode = current_size_mode;
                                        current_size_mode = current_size_mode.scale_down();
                                        if current_size_mode != old_size_mode {
                                            let size = current_size_mode.get_size();
                                            println!("Scaling window down to {}", current_size_mode.description());
                                            let new_size = LogicalSize::new(size as f64, size as f64);
                                            window.set_inner_size(new_size);
                                            window.request_redraw();
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                        winit::event::ElementState::Released => {
                            match keycode {
                                VirtualKeyCode::LControl | VirtualKeyCode::RControl => {
                                    ctrl_pressed = false;
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(physical_size),
                ..
            } => {
                // Nur loggen, wenn sich wirklich was ändert, um Spam zu vermeiden
                // println!("Window resized to: {}x{}", physical_size.width, physical_size.height);
                if let Err(e) = pixels.resize_surface(physical_size.width, physical_size.height) {
                    eprintln!("Failed to resize surface: {}", e);
                }
            }
            Event::RedrawRequested(_) => {
                if last_ram_check.elapsed() >= Duration::from_secs(1) {
                    system.refresh_memory();

                    let total_ram = system.total_memory();
                    let used_ram = system.used_memory();
                    current_ram_percent = if total_ram > 0 {
                        (used_ram as f64 / total_ram as f64) * 100.0
                    } else {
                        0.0
                    };

                    last_ram_check = Instant::now();
                }

                let animation_speed = if current_ram_percent <= 30.0 {
                    Duration::from_millis(200)
                } else if current_ram_percent <= 50.0 {
                    Duration::from_millis(150)
                } else if current_ram_percent <= 80.0 {
                    Duration::from_millis(100)
                } else {
                    Duration::from_millis(60)
                };

                let (sprite_file, color_name) = match current_ram_percent {
                    p if p <= 30.0 => ("lavalampe_green.png", "Green"),
                    p if p <= 50.0 => ("lavalampe_yellow.png", "Yellow"),
                    p if p <= 80.0 => ("lavalampe_orange.png", "Orange"),
                    _ => ("lavalampe_red.png", "Red"),
                };

                if current_sprite_file != sprite_file {
                    println!("=== Switching to {} lava ({:.1}% RAM used) ===", color_name, current_ram_percent);

                    match load_lava_animation(sprite_file) {
                        Some(new_anim) => {
                            println!("✓ Successfully loaded {}", sprite_file);
                            current_animation = Some(new_anim);
                            current_sprite_file = sprite_file;
                            frame_index = 0;
                        }
                        None => {
                            eprintln!("✗ Failed to load {}", sprite_file);
                            if sprite_file != "lavalampe_green.png" {
                                println!("Trying green as fallback...");
                                if let Some(fallback) = load_lava_animation("lavalampe_green.png") {
                                    println!("✓ Fallback to green successful");
                                    current_animation = Some(fallback);
                                    current_sprite_file = "lavalampe_green.png";
                                    frame_index = 0;
                                } else {
                                    current_animation = None;
                                    current_sprite_file = "";
                                }
                            } else {
                                current_animation = None;
                                current_sprite_file = "";
                            }
                        }
                    }
                }

                let frame = pixels.frame_mut();

                // Clear background
                for pixel in frame.chunks_exact_mut(4) {
                    pixel[0] = 0; pixel[1] = 0; pixel[2] = 0; pixel[3] = 255;
                }

                // Debug pattern if no animation
                if current_animation.is_none() {
                    let color = match current_ram_percent {
                        p if p <= 30.0 => [0, 255, 0, 255],
                        p if p <= 50.0 => [255, 255, 0, 255],
                        p if p <= 80.0 => [255, 165, 0, 255],
                        _ => [255, 0, 0, 255],
                    };
                    for pixel in frame.chunks_exact_mut(4) {
                        pixel.copy_from_slice(&color);
                    }
                }

                // Render animation
                if let Some((sprite_data, sprite_width, _)) = &current_animation {
                    let frames_available = *sprite_width / WINDOW_SIZE;
                    if frames_available > 0 && (*sprite_width % WINDOW_SIZE == 0) {
                        let actual_frame_count = frames_available.min(ANIMATION_FRAMES);

                        if last_update.elapsed() >= animation_speed {
                            frame_index = (frame_index + 1) % actual_frame_count;
                            last_update = Instant::now();
                        }

                        let frame_x_start = frame_index * WINDOW_SIZE;

                        for y in 0..WINDOW_SIZE {
                            for x in 0..WINDOW_SIZE {
                                let source_x = frame_x_start + x;
                                let source_index = (y * *sprite_width) + source_x;
                                let dest_index = (y * WINDOW_SIZE + x) * 4;

                                if source_index < sprite_data.len() {
                                    let source_pixel = sprite_data[source_index];
                                    let background = [
                                        frame[dest_index], frame[dest_index + 1],
                                        frame[dest_index + 2], frame[dest_index + 3]
                                    ];
                                    let blended = blend_alpha(background, source_pixel);
                                    frame[dest_index] = blended[0];
                                    frame[dest_index + 1] = blended[1];
                                    frame[dest_index + 2] = blended[2];
                                    frame[dest_index + 3] = blended[3];
                                }
                            }
                        }
                    }
                }

                if let Err(e) = pixels.render() {
                    eprintln!("pixels.render() failed: {}", e);
                    *control_flow = ControlFlow::Exit;
                }
            }
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            _ => {}
        }
    });
}