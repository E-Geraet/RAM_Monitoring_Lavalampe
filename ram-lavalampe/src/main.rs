use minifb::{Key, ScaleMode, Window, WindowOptions};
use image::{io::Reader as ImageReader, GenericImageView};
use sysinfo::{System, SystemExt};
use std::time::{Duration, Instant};
use std::path::Path;
use std::collections::HashSet;
use std::sync::Mutex;

// TODO: Maybe make these configurable later?
const WINDOW_SIZE: usize = 128;
const ANIMATION_FRAMES: usize = 90;

// Asset paths - probably should move these to a config file at some point
const GREEN_LAVA: &str = "assets/lavalampe_green.png";
const YELLOW_LAVA: &str = "assets/lavalampe_yellow.png"; 
const ORANGE_LAVA: &str = "assets/lavalampe_orange.png";
const RED_LAVA: &str = "assets/lavalampe_red.png";

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

fn load_lava_animation(file_path: &str) -> Option<(Vec<u32>, usize, usize)> {
    // Check if file exists first
    if !Path::new(file_path).exists() {
        warn_once(&format!("Can't find sprite sheet: {}", file_path));
        return None;
    }

    // Try to load the image
    let img = match ImageReader::open(file_path) {
        Ok(reader) => match reader.decode() {
            Ok(image) => image,
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

    // Convert pixels to our buffer format
    let mut pixel_data = vec![0u32; width * height];
    let has_alpha = img.color().has_alpha();

    for (x, y, pixel) in img.pixels() {
        let r = pixel[0] as u32;
        let g = pixel[1] as u32; 
        let b = pixel[2] as u32;
        let a = if pixel.0.len() > 3 && has_alpha {
            pixel[3] as u32
        } else {
            255  // Default to opaque
        };
        
        let index = (y as usize * width) + x as usize;
        pixel_data[index] = (a << 24) | (r << 16) | (g << 8) | b;
    }

    Some((pixel_data, width, height))
}

fn main() {
    println!("Starting RAM Lava Lamp...");
    
    let mut system = System::new_all();
    
    // Create the window - small and borderless for that desktop widget feel
    let mut window = match Window::new(
        "RAM Lava Lamp",
        WINDOW_SIZE,
        WINDOW_SIZE,
        WindowOptions {
            resize: false,
            scale_mode: ScaleMode::UpperLeft,
            borderless: true,
            ..WindowOptions::default()
        },
    ) {
        Ok(win) => win,
        Err(e) => panic!("Couldn't create window: {}", e),
    };

    // Animation state
    let mut current_animation: Option<(Vec<u32>, usize, usize)> = None;
    let mut current_sprite_file = "";
    let mut frame_buffer = vec![0u32; WINDOW_SIZE * WINDOW_SIZE];
    let mut frame_index = 0;
    let mut last_update = Instant::now();

    system.refresh_memory();
    print_once("RAM monitoring started");

    // Main loop
    while window.is_open() && !window.is_key_down(Key::Escape) {
        system.refresh_memory();
        
        let total_ram = system.total_memory();
        let used_ram = system.used_memory();
        let ram_percent = if total_ram > 0 {
            (used_ram as f64 / total_ram as f64) * 100.0
        } else {
            0.0
        };

        // Different animation speeds based on RAM usage
        let animation_speed = if ram_percent <= 30.0 {
            Duration::from_millis(200)  // Slow and relaxed
        } else if ram_percent <= 50.0 {
            Duration::from_millis(150)  // Getting busier
        } else if ram_percent <= 80.0 {
            Duration::from_millis(100)  // Pretty busy
        } else {
            Duration::from_millis(60)   // Frantic!
        };

        // Pick the right color based on RAM usage
        let (sprite_file, color_name) = match ram_percent {
            p if p <= 30.0 => (GREEN_LAVA, "Green"),
            p if p <= 50.0 => (YELLOW_LAVA, "Yellow"), 
            p if p <= 80.0 => (ORANGE_LAVA, "Orange"),
            _ => (RED_LAVA, "Red"),
        };

        // Load new animation if needed
        if current_sprite_file != sprite_file {
            print_once(&format!("Switching to {} lava ({:.1}% RAM)", color_name, ram_percent));
            
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

                // Copy the current frame to our display buffer
                let frame_x_start = frame_index * WINDOW_SIZE;
                
                for y in 0..WINDOW_SIZE {
                    for x in 0..WINDOW_SIZE {
                        let source_x = frame_x_start + x;
                        let source_index = (y * *sprite_width) + source_x;
                        let dest_index = y * WINDOW_SIZE + x;
                        
                        if source_index < sprite_data.len() {
                            frame_buffer[dest_index] = sprite_data[source_index];
                        } else {
                            // Shouldn't happen, but just in case
                            frame_buffer[dest_index] = 0xFF00FF; // Magenta error
                        }
                    }
                }
            } else {
                warn_once(&format!("Bad sprite format: width {}", sprite_width));
                // Fill with red to show there's an error
                frame_buffer.fill(0xFF0000);
            }
        } else {
            // No animation loaded - just show black
            frame_buffer.fill(0);
        }

        // Update the window
        if let Err(e) = window.update_with_buffer(&frame_buffer, WINDOW_SIZE, WINDOW_SIZE) {
            eprintln!("Window update failed: {}", e);
            break;
        }

        // Don't peg the CPU - sleep a bit
        let min_sleep = Duration::from_millis(10);
        let time_since_frame = last_update.elapsed();
        
        if time_since_frame < animation_speed {
            let sleep_time = (animation_speed - time_since_frame).min(min_sleep);
            if sleep_time > Duration::ZERO {
                std::thread::sleep(sleep_time);
            }
        } else {
            std::thread::sleep(Duration::from_millis(1));
        }
    }
    
    println!("Shutting down...");
}
