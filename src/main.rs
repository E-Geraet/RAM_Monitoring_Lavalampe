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
use std::path::Path;
use std::collections::HashSet;
use std::sync::Mutex;

// TODO: Maybe make these configurable later?
const WINDOW_SIZE: usize = 128;
const ANIMATION_FRAMES: usize = 200;

// Asset paths - probably should move these to a config file at some point
const GREEN_LAVA: &str = "assets/lavalampe_green.png";
const YELLOW_LAVA: &str = "assets/lavalampe_yellow.png";
const ORANGE_LAVA: &str = "assets/lavalampe_orange.png";
const RED_LAVA: &str = "assets/lavalampe_red.png";

// Window size modes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum WindowSizeMode {
    Small,   // 128x128
    Medium,  // 256x256
    Large,   // 512x512
}

impl WindowSizeMode {
    fn get_size(&self) -> usize {
        match self {
            WindowSizeMode::Small => WINDOW_SIZE,
            WindowSizeMode::Medium => WINDOW_SIZE * 2,
            WindowSizeMode::Large => WINDOW_SIZE * 4,
        }
    }

    fn scale_up(&self) -> WindowSizeMode {
        match self {
            WindowSizeMode::Small => WindowSizeMode::Medium,
            WindowSizeMode::Medium => WindowSizeMode::Large,
            WindowSizeMode::Large => WindowSizeMode::Large, // Stays large
        }
    }

    fn scale_down(&self) -> WindowSizeMode {
        match self {
            WindowSizeMode::Small => WindowSizeMode::Small, // Stays small
            WindowSizeMode::Medium => WindowSizeMode::Small,
            WindowSizeMode::Large => WindowSizeMode::Medium,
        }
    }

    fn description(&self) -> &'static str {
        match self {
            WindowSizeMode::Small => "128x128",
            WindowSizeMode::Medium => "256x256",
            WindowSizeMode::Large => "512x512",
        }
    }
}

// Simple cache for print messages so we don't spam the console
lazy_static::lazy_static! {
    static ref ALREADY_PRINTED: Mutex<HashSet<String>> = Mutex::new(HashSet::new());
}

// Helper to print messages only once
fn print_once(msg: &str) {
    let mut cache = ALREADY_PRINTED.lock().unwrap();
    if !cache.contains(msg) {
        println!("{}", msg);
        cache.insert(msg.to_string());
    }
}

fn warn_once(msg: &str) {
    let mut cache = ALREADY_PRINTED.lock().unwrap();
    if !cache.contains(msg) {
        eprintln!("Warning: {}", msg);
        cache.insert(msg.to_string());
    }
}

fn load_lava_animation(file_path: &str) -> Option<(Vec<Rgba<u8>>, usize, usize)> {
    // Check if file exists first
    if !Path::new(file_path).exists() {
        warn_once(&format!("Can't find sprite sheet: {}", file_path));
        return None;
    }

    // Try to load the image
    let img = match ImageReader::open(file_path) {
        Ok(reader) => match reader.decode() {
            Ok(image) => image.to_rgba8(), // Convert to RGBA8 for consistent handling
            Err(e) => {
                warn_once(&format!("Failed to decode {}: {}", file_path, e));
                return None;
            }
        },
        Err(e) => {
            warn_once(&format!("Can't open {}: {}", file_path, e));
            return None;
        }
    };

    let (width, height) = img.dimensions();
    let width = width as usize;
    let height = height as usize;

    // Basic sanity checks
    if height != WINDOW_SIZE {
        warn_once(&format!("Wrong height for {}: got {}, expected {}",
                          file_path, height, WINDOW_SIZE));
        return None;
    }

    if width < WINDOW_SIZE {
        warn_once(&format!("Image too narrow: {}", file_path));
        return None;
    }

    // Convert pixels to RGBA format
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

    if fg_a == 0 {
        return background;
    }

    if fg_a == 255 {
        return [fg_r, fg_g, fg_b, 255];
    }

    // Alpha blending formula
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
    println!("Controls: Ctrl + Up Arrow = Scale Up, Ctrl + Down Arrow = Scale Down, Esc = Exit");

    let mut system = System::new_all();

    // Create event loop and window
    let event_loop = EventLoop::new();
    let window = {
        let size = LogicalSize::new(WINDOW_SIZE as f64, WINDOW_SIZE as f64);
        WindowBuilder::new()
            .with_title("RAM Lava Lamp")
            .with_inner_size(size)
            .with_min_inner_size(LogicalSize::new(WINDOW_SIZE as f64, WINDOW_SIZE as f64))
            .with_max_inner_size(LogicalSize::new((WINDOW_SIZE * 4) as f64, (WINDOW_SIZE * 4) as f64)) // Allow up to 4x size
            .with_resizable(true) // Make it resizable so we can change the size
            .with_decorations(false) // Borderless for widget feel
            .build(&event_loop)?
    };

    // Initialize pixels with scaling enabled
    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        // Create pixels buffer - this will automatically scale to fit the window
        Pixels::new(WINDOW_SIZE as u32, WINDOW_SIZE as u32, surface_texture)?
    };

    // Window size state tracking
    let mut current_size_mode = WindowSizeMode::Small;
    let mut ctrl_pressed = false; // State for Ctrl key

    // Animation state
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
                                    if ctrl_pressed { // Check if Ctrl is pressed
                                        let old_size_mode = current_size_mode;
                                        current_size_mode = current_size_mode.scale_up();
                                        if current_size_mode != old_size_mode { // Only update if size changed
                                            let size = current_size_mode.get_size();
                                            println!("Scaling window up to {}", current_size_mode.description());
                                            let new_size = LogicalSize::new(size as f64, size as f64);
                                            window.set_inner_size(new_size);
                                            window.request_redraw();
                                        }
                                    }
                                }
                                VirtualKeyCode::Down => {
                                    if ctrl_pressed { // Check if Ctrl is pressed
                                        let old_size_mode = current_size_mode;
                                        current_size_mode = current_size_mode.scale_down();
                                        if current_size_mode != old_size_mode { // Only update if size changed
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
                // Handle window resize
                println!("Window resized to: {}x{}", physical_size.width, physical_size.height);
                if let Err(e) = pixels.resize_surface(physical_size.width, physical_size.height) {
                    eprintln!("Failed to resize surface: {}", e);
                }
                // The pixels buffer will automatically scale the 128x128 content to fill the new window size
            }
            Event::RedrawRequested(_) => {
                // Check RAM usage (but not too frequently)
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

                // Different animation speeds based on RAM usage
                let animation_speed = if current_ram_percent <= 30.0 {
                    Duration::from_millis(200)  // Slow and relaxed
                } else if current_ram_percent <= 50.0 {
                    Duration::from_millis(150)  // Getting busier
                } else if current_ram_percent <= 80.0 {
                    Duration::from_millis(100)  // Pretty busy
                } else {
                    Duration::from_millis(60)   // Frantic!
                };

                // Pick the right color based on RAM usage
                let (sprite_file, color_name) = match current_ram_percent {
                    p if p <= 30.0 => (GREEN_LAVA, "Green"),
                    p if p <= 50.0 => (YELLOW_LAVA, "Yellow"),
                    p if p <= 80.0 => (ORANGE_LAVA, "Orange"),
                    _ => (RED_LAVA, "Red"),
                };

                // Load new animation if needed
                if current_sprite_file != sprite_file {
                    print_once(&format!("Switching to {} lava ({:.1}% RAM)", color_name, current_ram_percent));

                    if let Some(new_anim) = load_lava_animation(sprite_file) {
                        current_animation = Some(new_anim);
                        current_sprite_file = sprite_file;
                        frame_index = 0;
                        print_once(&format!("Loaded {}", sprite_file));
                    } else {
                        warn_once(&format!("Failed to load {}", sprite_file));

                        // Try falling back to green if it's not what we were trying to load
                        if sprite_file != GREEN_LAVA {
                            print_once("Trying green as fallback...");
                            if let Some(fallback) = load_lava_animation(GREEN_LAVA) {
                                current_animation = Some(fallback);
                                current_sprite_file = GREEN_LAVA;
                                frame_index = 0;
                                print_once("Fallback successful");
                            } else {
                                warn_once("Even green fallback failed!");
                                current_animation = None;
                                current_sprite_file = "";
                            }
                        }
                    }
                }

                // Get the pixel buffer
                let frame = pixels.frame_mut();

                // Clear background to black with full alpha
                for pixel in frame.chunks_exact_mut(4) {
                    pixel[0] = 0;   // R
                    pixel[1] = 0;   // G
                    pixel[2] = 0;   // B
                    pixel[3] = 255; // A
                }

                // Animate if we have data
                if let Some((sprite_data, sprite_width, _)) = &current_animation {
                    let frames_available = *sprite_width / WINDOW_SIZE;

                    if frames_available > 0 && (*sprite_width % WINDOW_SIZE == 0) {
                        // Figure out how many frames we actually have vs expect
                        let actual_frame_count = if frames_available < ANIMATION_FRAMES {
                            print_once(&format!("Only {} frames available (expected {})",
                                              frames_available, ANIMATION_FRAMES));
                            frames_available
                        } else if frames_available > ANIMATION_FRAMES {
                            print_once(&format!("Extra frames found: {} (using {})",
                                              frames_available, ANIMATION_FRAMES));
                            ANIMATION_FRAMES
                        } else {
                            ANIMATION_FRAMES
                        };

                        // Advance frame if enough time has passed
                        if last_update.elapsed() >= animation_speed {
                            frame_index = (frame_index + 1) % actual_frame_count;
                            last_update = Instant::now();
                        }

                        // Copy the current frame to our display buffer with alpha blending
                        let frame_x_start = frame_index * WINDOW_SIZE;

                        for y in 0..WINDOW_SIZE {
                            for x in 0..WINDOW_SIZE {
                                let source_x = frame_x_start + x;
                                let source_index = (y * *sprite_width) + source_x;
                                let dest_index = (y * WINDOW_SIZE + x) * 4;

                                if source_index < sprite_data.len() {
                                    let source_pixel = sprite_data[source_index];
                                    let background = [
                                        frame[dest_index],
                                        frame[dest_index + 1],
                                        frame[dest_index + 2],
                                        frame[dest_index + 3]
                                    ];

                                    let blended = blend_alpha(background, source_pixel);

                                    frame[dest_index] = blended[0];     // R
                                    frame[dest_index + 1] = blended[1]; // G
                                    frame[dest_index + 2] = blended[2]; // B
                                    frame[dest_index + 3] = blended[3]; // A
                                } else {
                                    // Shouldn't happen, but just in case - magenta error
                                    frame[dest_index] = 255;     // R
                                    frame[dest_index + 1] = 0;   // G
                                    frame[dest_index + 2] = 255; // B
                                    frame[dest_index + 3] = 255; // A
                                }
                            }
                        }
                    } else {
                        warn_once(&format!("Bad sprite format: width {}", sprite_width));
                        // Fill with red to show there's an error
                        for pixel in frame.chunks_exact_mut(4) {
                            pixel[0] = 255; // R
                            pixel[1] = 0;   // G
                            pixel[2] = 0;   // B
                            pixel[3] = 255; // A
                        }
                    }
                }

                // Render the frame
                if let Err(e) = pixels.render() {
                    eprintln!("pixels.render() failed: {}", e);
                    *control_flow = ControlFlow::Exit;
                }
            }
            Event::MainEventsCleared => {
                // Request redraw
                window.request_redraw();
            }
            _ => {}
        }
    });
}
