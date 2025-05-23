use minifb::{Key, ScaleMode, Window, WindowOptions};
use image::{io::Reader as ImageReader, GenericImageView};

const WIDTH: usize = 128;
const HEIGHT: usize = 128;
const IMAGE_PATH: &str = "assets/lavalampe_rot.png";

fn main() {
    let img = match ImageReader::open(IMAGE_PATH) {
        Ok(reader) => match reader.decode() {
            Ok(img) => img,
            Err(err) => {
                eprintln!("Error decoding image: {}", err);
                return;
            }
        },
        Err(err) => {
            eprintln!("Error opening image: {}", err);
            return;
        }
    };

    let (image_width_loaded, image_height_loaded) = img.dimensions();
    
    let mut image_buffer: Vec<u32> = vec![0; (image_width_loaded * image_height_loaded) as usize];
    let color_type = img.color();

    for (x, y, pixel) in img.pixels() {
        let r = pixel[0] as u32;
        let g = pixel[1] as u32;
        let b = pixel[2] as u32;
        let a = if color_type.has_alpha() { pixel[3] as u32 } else { 0xFF };
        image_buffer[(y * image_width_loaded + x) as usize] = (a << 24) | (r << 16) | (g << 8) | b;
    }

    let mut window = match Window::new(
        "RAM Lava Lamp",
        WIDTH,
        HEIGHT,
        WindowOptions {
            resize: false,
            scale_mode: ScaleMode::UpperLeft,
            ..WindowOptions::default()
        },
    ) {
        Ok(win) => win,
        Err(err) => {
            eprintln!("Unable to create window: {}", err);
            return;
        }
    };

    // Limit to max ~60 fps update rate
    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    let mut display_buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];
    for y_win in 0..HEIGHT {
        for x_win in 0..WIDTH {
            if y_win < image_height_loaded as usize && x_win < image_width_loaded as usize {
                let src_idx = (y_win * image_width_loaded as usize + x_win) as usize;
                display_buffer[y_win * WIDTH + x_win] = image_buffer[src_idx];
            }
        }
    }

    while window.is_open() && !window.is_key_down(Key::Escape) {
        if let Err(err) = window.update_with_buffer(&display_buffer, WIDTH, HEIGHT) {
            eprintln!("Window update error: {}", err);
            return;
        }
    }
}
