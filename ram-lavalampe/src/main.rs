use minifb::{Key, ScaleMode, Window, WindowOptions};
use image::{io::Reader as ImageReader, GenericImageView};
use sysinfo::{System, SystemExt};
use std::time::{Duration, Instant};
use std::path::Path;
use std::collections::HashSet;
use std::sync::Mutex;

// Fenster- und Frame-Dimensionen
const FRAME_WIDTH: usize = 128;
const FRAME_HEIGHT: usize = 128;
const NUM_FRAMES: usize = 90; // Erwartete Anzahl der Frames pro Sprite-Sheet

// Pfade zu den Sprite-Sheets
const SPRITE_PATH_GREEN: &str = "assets/lavalampe_green.png";
const SPRITE_PATH_YELLOW: &str = "assets/lavalampe_yellow.png";
const SPRITE_PATH_ORANGE: &str = "assets/lavalampe_orange.png";
const SPRITE_PATH_RED: &str = "assets/lavalampe_red.png";

lazy_static::lazy_static! {
    static ref PRINTED_MESSAGES: Mutex<HashSet<String>> = Mutex::new(HashSet::new());
}

fn println_once(message: &str) {
    let mut printed_messages = PRINTED_MESSAGES.lock().unwrap();
    if !printed_messages.contains(message) {
        println!("{}", message);
        printed_messages.insert(message.to_string());
    }
}
fn eprintln_once(message: &str) {
    let mut printed_messages = PRINTED_MESSAGES.lock().unwrap();
    if !printed_messages.contains(message) {
        eprintln!("{}", message);
        printed_messages.insert(message.to_string());
    }
}

fn load_sprite_sheet(path_str: &str) -> Option<(Vec<u32>, usize, usize)> {
    if !Path::new(path_str).exists() {
        eprintln_once(&format!("WARNUNG: Sprite-Sheet nicht gefunden unter '{}'", path_str));
        return None;
    }
    match ImageReader::open(path_str) {
        Ok(reader) => match reader.decode() {
            Ok(img) => {
                let (sheet_width_u32, sheet_height_u32) = img.dimensions();
                let sheet_width = sheet_width_u32 as usize;
                let sheet_height = sheet_height_u32 as usize;

                if sheet_height != FRAME_HEIGHT {
                    eprintln_once(&format!(
                        "FEHLER: Sprite-Sheet '{}' Höhe ({}) entspricht nicht der FRAME_HEIGHT ({}).",
                        path_str, sheet_height, FRAME_HEIGHT
                    ));
                    return None;
                }
                if sheet_width < FRAME_WIDTH {
                    eprintln_once(&format!(
                        "FEHLER: Sprite-Sheet '{}' Breite ({}) ist schmaler als ein einzelner Frame ({}).",
                        path_str, sheet_width, FRAME_WIDTH
                    ));
                    return None;
                }

                let mut buffer: Vec<u32> = vec![0; sheet_width * sheet_height];
                let color_type = img.color();

                for (x, y, pixel) in img.pixels() {
                    let r = pixel[0] as u32;
                    let g = pixel[1] as u32;
                    let b = pixel[2] as u32;
                    let a = if pixel.0.len() > 3 && color_type.has_alpha() {
                        pixel[3] as u32
                    } else {
                        0xFF
                    };
                    buffer[(y as usize * sheet_width) + x as usize] = (a << 24) | (r << 16) | (g << 8) | b;
                }
                Some((buffer, sheet_width, sheet_height))
            }
            Err(err) => {
                eprintln_once(&format!("FEHLER beim Dekodieren des Bildes '{}': {}", path_str, err));
                None
            }
        },
        Err(err) => {
            eprintln_once(&format!("FEHLER beim Öffnen des Bildes '{}': {}", path_str, err));
            None
        }
    }
}

fn main() {
    let mut sys = System::new_all();

    let mut window = Window::new(
        "RAM Lava Lampe",
        FRAME_WIDTH,
        FRAME_HEIGHT,
        WindowOptions {
            resize: false,
            scale_mode: ScaleMode::UpperLeft,
            borderless: true,
            ..WindowOptions::default()
        },
    )
        .unwrap_or_else(|e| {
            panic!("Fenster konnte nicht erstellt werden: {}", e);
        });

    let mut current_sprite_sheet_data: Option<(Vec<u32>, usize, usize)> = None;
    let mut current_sprite_path_str = "";
    let mut display_buffer: Vec<u32> = vec![0; FRAME_WIDTH * FRAME_HEIGHT];
    let mut current_frame_index: usize = 0;
    let mut last_frame_time = Instant::now();

    sys.refresh_memory();


    while window.is_open() && !window.is_key_down(Key::Escape) {
        sys.refresh_memory();
        let total_ram = sys.total_memory();
        let used_ram = sys.used_memory();
        let ram_usage_percent = if total_ram > 0 {
            (used_ram as f64 / total_ram as f64) * 100.0
        } else {
            0.0
        };

        let current_animation_delay = Duration::from_millis(if ram_usage_percent <= 30.0 {
            200
        } else if ram_usage_percent <= 50.0 {
            150
        } else if ram_usage_percent <= 80.0 {
            100
        } else {
            60
        });

        let (target_sprite_path_str, target_color_name) = if ram_usage_percent <= 30.0 {
            (SPRITE_PATH_GREEN, "Grün")
        } else if ram_usage_percent <= 50.0 {
            (SPRITE_PATH_YELLOW, "Gelb")
        } else if ram_usage_percent <= 80.0 {
            (SPRITE_PATH_ORANGE, "Orange")
        } else {
            (SPRITE_PATH_RED, "Rot")
        };

        if current_sprite_path_str != target_sprite_path_str {
            println_once(&format!("Benötigtes Sprite-Sheet: {} ({})", target_color_name, target_sprite_path_str));
            let new_data = load_sprite_sheet(target_sprite_path_str);

            if new_data.is_some() {
                let (buf, w, h) = new_data.unwrap();
                current_sprite_sheet_data = Some((buf, w, h));
                current_sprite_path_str = target_sprite_path_str;
                current_frame_index = 0;
                println_once(&format!("Erfolgreich geladen: {}.", target_sprite_path_str));
            } else {
                eprintln_once(&format!("Fehler beim Laden von: {}.", target_sprite_path_str));
                if target_sprite_path_str != SPRITE_PATH_GREEN && current_sprite_path_str != SPRITE_PATH_GREEN {
                    println_once(&format!("Versuche Fallback auf GRÜN: {}", SPRITE_PATH_GREEN));
                    let fallback_data = load_sprite_sheet(SPRITE_PATH_GREEN);
                    if fallback_data.is_some() {
                        let (buf, w, h) = fallback_data.unwrap();
                        current_sprite_sheet_data = Some((buf, w, h));
                        current_sprite_path_str = SPRITE_PATH_GREEN;
                        current_frame_index = 0;
                        println_once(&format!("Erfolgreich Fallback geladen: {}.", SPRITE_PATH_GREEN));
                    } else {
                        eprintln_once("KRITISCH: Fallback GRÜN konnte auch nicht geladen werden.");
                        current_sprite_sheet_data = None;
                        current_sprite_path_str = "";
                    }
                } else if target_sprite_path_str == SPRITE_PATH_GREEN {
                    eprintln_once("KRITISCH: Ziel-Sprite GRÜN konnte nicht geladen werden.");
                    current_sprite_sheet_data = None;
                    current_sprite_path_str = "";
                }
            }
        }

        if let Some((sheet_buffer, sheet_width, _sheet_height)) = &current_sprite_sheet_data {
            let actual_frames_in_sheet = *sheet_width / FRAME_WIDTH;

            if actual_frames_in_sheet > 0 && (*sheet_width % FRAME_WIDTH == 0) {
                let effective_num_frames = if actual_frames_in_sheet < NUM_FRAMES {
                    println_once(&format!("WARNUNG: Sprite-Sheet '{}' hat nur {} Frames (Breite: {}), erwartet wurden {}. Animation wird mit {} Frames abgespielt.", current_sprite_path_str, actual_frames_in_sheet, sheet_width, NUM_FRAMES, actual_frames_in_sheet));
                    actual_frames_in_sheet
                } else if actual_frames_in_sheet > NUM_FRAMES {
                    println_once(&format!("WARNUNG: Sprite-Sheet '{}' hat {} Frames (Breite: {}), mehr als die erwarteten {}. Animation wird mit {} Frames (global NUM_FRAMES) abgespielt.", current_sprite_path_str, actual_frames_in_sheet, sheet_width, NUM_FRAMES, NUM_FRAMES));
                    NUM_FRAMES
                } else {
                    NUM_FRAMES
                };

                if last_frame_time.elapsed() >= current_animation_delay {
                    current_frame_index = (current_frame_index + 1) % effective_num_frames;
                    last_frame_time = Instant::now();
                }

                let frame_x_offset = current_frame_index * FRAME_WIDTH;

                for y_win in 0..FRAME_HEIGHT {
                    // HIER WAR DER FEHLER, WIDTH wurde zu FRAME_WIDTH korrigiert:
                    for x_win in 0..FRAME_WIDTH {
                        let src_x = frame_x_offset + x_win;
                        let src_idx = (y_win * *sheet_width) + src_x;
                        let display_idx = y_win * FRAME_WIDTH + x_win;

                        if src_idx < sheet_buffer.len() {
                            display_buffer[display_idx] = sheet_buffer[src_idx];
                        } else {
                            display_buffer[display_idx] = 0xFFFF00FF; // Magenta bei Indexfehler
                        }
                    }
                }
            } else {
                eprintln_once(&format!("FEHLER: Sprite-Sheet '{}' (Breite: {}) ist nicht korrekt für Animation formatiert.", current_sprite_path_str, sheet_width));
                display_buffer.iter_mut().for_each(|p| *p = 0xFFFF0000); // Rot als Fehleranzeige
            }
        } else {
            display_buffer.iter_mut().for_each(|p| *p = 0);
        }

        if let Err(err) = window.update_with_buffer(&display_buffer, FRAME_WIDTH, FRAME_HEIGHT) {
            eprintln!("Fehler beim Fenster-Update: {}", err);
            break;
        }

        let loop_min_duration = Duration::from_millis(10);
        let elapsed_since_last_frame = last_frame_time.elapsed();

        if elapsed_since_last_frame < current_animation_delay {
            let time_to_next_frame = current_animation_delay - elapsed_since_last_frame;
            let sleep_duration = time_to_next_frame.min(loop_min_duration);
            if sleep_duration > Duration::ZERO {
                std::thread::sleep(sleep_duration);
            }
        } else {
            std::thread::sleep(Duration::from_millis(1).min(loop_min_duration));
        }
    }
}