// Cannot change CLEAR_COLOR when using fade due to problems with the alpha channel

use rand::Rng;

use pixels::{Pixels, SurfaceTexture};
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

const WIDTH: usize = 256;
const HEIGHT: usize = 64;
const PIXEL_SIZE: usize = 8;

// Dark grey
// const CLEAR_COLOR: (u8, u8, u8, u8) = (0x07, 0x11, 0x08, 0x00);
// Teal
const CLEAR_COLOR: (u8, u8, u8, u8) = (0x0E, 0x7C, 0x7B, 0x00);

// Light pink
// const STAR_COLOR: (u8, u8, u8, u8) = (0xBF, 0xB1, 0xC1, 0xFF);
// Light blue
// const STAR_COLOR: (u8, u8, u8, u8) = (0xC7, 0xDB, 0xE6, 0xFF);
// Bright orange
// const STAR_COLOR: (u8, u8, u8, u8) = (0xEF, 0x83, 0x54, 0xFF);
// Red
const STAR_COLOR: (u8, u8, u8, u8) = (0xD6, 0x22, 0x46, 0xFF);

fn main() {
    // Window parameters
    let title = "Starfield";

    // Init window
    let event_loop = EventLoop::new();

    let window_width = WIDTH * PIXEL_SIZE;
    let window_height = HEIGHT * PIXEL_SIZE;
    let size = LogicalSize::new(window_width as f64, window_height as f64);

    let window = WindowBuilder::new()
        .with_title(title)
        .with_inner_size(size)
        .with_min_inner_size(size)
        .build(&event_loop)
        .unwrap();

    // Init pixels
    let window_size = window.inner_size();
    let mut pixels = Pixels::new(
        WIDTH as u32,
        HEIGHT as u32,
        SurfaceTexture::new(window_size.width, window_size.height, &window),
    )
    .unwrap();

    // Init starfield
    let num_stars = 500;
    let trail_length = 20.0;
    let speed_min = 0.5;
    let speed_max = 2.0;

    let mut rng = rand::thread_rng();
    let mut starfield_right = Vec::with_capacity(num_stars);
    for _ in 0..num_stars {
        let x = rng.gen_range(0.0..WIDTH as f64);
        let y = rng.gen_range(0.0..HEIGHT as f64);
        let direction = (1.0, 0.0);
        let speed = rng.gen_range(speed_min..speed_max);
        starfield_right.push(Star {
            position: (x, y),
            direction,
            speed,
        });
    }

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent {
                window_id,
                event: WindowEvent::CloseRequested,
            } => {
                // Close window
                if window_id == window.id() {
                    *control_flow = ControlFlow::Exit;
                }
            }
            Event::MainEventsCleared => {
                // Update
                let width = WIDTH as f64;
                for star in starfield_right.iter_mut() {
                    // Star
                    let (x0, y0) = star.position;
                    let (dx, dy) = star.direction;
                    let speed = star.speed;

                    // Reset star if it is completely off screen
                    if x0 >= width && x0 - dx * speed * trail_length >= width {
                        star.position = (0.0, rng.gen_range(0.0..HEIGHT as f64));
                        star.speed = rng.gen_range(speed_min..speed_max);
                    } else {
                        star.position = (x0 + dx * speed, y0 + dy * speed);
                    }
                }

                window.request_redraw();
            }
            Event::RedrawRequested(_) => {
                // Render
                clear(CLEAR_COLOR, pixels.frame_mut());
                for star in starfield_right.iter() {
                    let (x, y) = star.position;
                    let (dx, dy) = star.direction;
                    let speed = star.speed;

                    let x0 = x as i32;
                    let y0 = y as i32;
                    let x1 = (x - dx * trail_length * speed) as i32;
                    let y1 = (y - dy * trail_length * speed) as i32;

                    draw_line(x0, y0, x1, y1, STAR_COLOR, pixels.frame_mut(), true);
                }

                pixels.render().unwrap();
            }
            _ => {}
        }
    });
}

struct Star {
    position: (f64, f64),
    direction: (f64, f64),
    speed: f64,
}

fn clear(color: (u8, u8, u8, u8), frame: &mut [u8]) {
    debug_assert_eq!(frame.len(), 4 * WIDTH as usize * HEIGHT as usize);

    for pixel in frame.chunks_exact_mut(4) {
        pixel.copy_from_slice(&[color.0, color.1, color.2, color.3]);
    }
}

fn draw_line(
    x0: i32,
    y0: i32,
    x1: i32,
    y1: i32,
    color: (u8, u8, u8, u8),
    frame: &mut [u8],
    fade: bool,
) {
    debug_assert_eq!(frame.len(), 4 * WIDTH * HEIGHT);

    let dx = (x1 - x0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let dy = -(y1 - y0).abs();
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut error = dx + dy;

    let width = WIDTH as i32;
    let height = HEIGHT as i32;

    // Used for fading
    let d = (dx * dx + dy * dy) as f64;
    let a = STAR_COLOR.3 as f64;

    let mut x = x0;
    let mut y = y0;

    loop {
        // Draw pixel
        if x >= 0 && x < width && y >= 0 && y < height {
            let pixel_index = ((y as usize * WIDTH) + x as usize) * 4;
            let pixel = &mut frame[pixel_index..pixel_index + 4];

            if fade {
                let alpha = a * ((x1 - x) * (x1 - x) + (y1 - y) * (y1 - y)) as f64 / d;
                pixel.copy_from_slice(&[
                    color.0,
                    color.1,
                    color.2,
                    pixel[3].saturating_add(alpha as u8),
                ]);
            } else {
                pixel.copy_from_slice(&[color.0, color.1, color.2, color.3]);
            }
        }

        // Move to next pixel
        if x == x1 && y == y1 {
            break;
        }

        if error * 2 >= dy {
            if x == x1 {
                break;
            }
            error += dy;
            x += sx;
        }
        if error * 2 <= dx {
            if y == y1 {
                break;
            }
            error += dx;
            y += sy;
        }
    }
}
