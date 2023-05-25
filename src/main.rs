use pixels::{Pixels, SurfaceTexture};
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

const WINDOW_TITLE: &str = "Plotter";
const SCREEN_WIDTH: usize = 128;
const SCREEN_HEIGHT: usize = 128;
const PIXEL_SIZE: usize = 8;

const SCREEN_CENTER_X: f64 = (SCREEN_WIDTH / 2) as f64;
const SCREEN_CENTER_Y: f64 = (SCREEN_HEIGHT / 2) as f64;

fn main() {
    let event_loop = EventLoop::new();

    let window_size = LogicalSize::new(
        (SCREEN_WIDTH * PIXEL_SIZE) as f64,
        (SCREEN_HEIGHT * PIXEL_SIZE) as f64,
    );

    let window = WindowBuilder::new()
        .with_title(WINDOW_TITLE)
        .with_inner_size(window_size)
        .with_min_inner_size(window_size)
        .build(&event_loop)
        .expect("Failed to create window");

    let mut pixels = Pixels::new(
        SCREEN_WIDTH as u32,
        SCREEN_HEIGHT as u32,
        SurfaceTexture::new(
            window.inner_size().width,
            window.inner_size().height,
            &window,
        ),
    )
    .expect("Failed to create pixel buffer");

    let mut offset_x = -SCREEN_CENTER_X;
    let mut offset_y = -SCREEN_CENTER_Y;
    let mut scale_x = 1.0;
    let mut scale_y = 1.0;

    let circle_x: f64 = 0.0;
    let circle_y: f64 = 0.0;
    let circle_radius: f64 = 32.0;

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent {
                window_id,
                event: WindowEvent::CloseRequested,
            } => {
                if window_id == window.id() {
                    *control_flow = ControlFlow::Exit;
                }
            }
            Event::MainEventsCleared => {
                // Update
                let (world_x_before_zoom, world_y_before_zoom) = screen_to_world(
                    SCREEN_CENTER_X,
                    SCREEN_CENTER_Y,
                    offset_x,
                    offset_y,
                    scale_x,
                    scale_y,
                );

                scale_x *= 1.005;
                scale_y *= 1.005;

                let (world_x_after_zoom, world_y_after_zoom) = screen_to_world(
                    SCREEN_CENTER_X,
                    SCREEN_CENTER_Y,
                    offset_x,
                    offset_y,
                    scale_x,
                    scale_y,
                );

                offset_x += world_x_before_zoom - world_x_after_zoom;
                offset_y += world_y_before_zoom + world_y_after_zoom;

                window.request_redraw();
            }
            Event::RedrawRequested(_) => {
                // Render
                let frame = pixels.frame_mut();
                clear(frame);

                // Draw circle
                for row in 0..SCREEN_HEIGHT {
                    for col in 0..SCREEN_WIDTH {
                        let (x, y) = screen_to_world(
                            col as f64, row as f64, offset_x, offset_y, scale_x, scale_y,
                        );

                        if (x - circle_x).powi(2) + (y - circle_y).powi(2) >= circle_radius.powi(2)
                        {
                            continue;
                        }

                        if col >= SCREEN_WIDTH || row >= SCREEN_HEIGHT {
                            continue;
                        }

                        let pixel_index = ((col as usize * SCREEN_WIDTH) + row as usize) * 4;
                        let pixel = &mut frame[pixel_index..pixel_index + 4];

                        pixel.copy_from_slice(&[0, 255, 0, 255]);
                    }
                }

                // Draw sin
                for col in 0..SCREEN_WIDTH {
                    let (x, _) =
                        screen_to_world(col as f64, 0.0, offset_x, offset_y, scale_x, scale_y);

                    let (_, y) = world_to_screen(x, x.sin(), offset_x, offset_y, scale_x, scale_y);
                    let y = y.round();
                    if y < 0.0 || y >= SCREEN_HEIGHT as f64 {
                        continue;
                    }

                    let pixel_index = ((y as usize * SCREEN_WIDTH) + col) * 4;
                    let pixel = &mut frame[pixel_index..pixel_index + 4];

                    pixel.copy_from_slice(&[255, 0, 0, 255]);
                }

                // Draw line
                let (x0, y0) = world_to_screen(0.0, 0.0, offset_x, offset_y, scale_x, scale_y);
                let (x1, y1) = world_to_screen(32.0, 32.0, offset_x, offset_y, scale_x, scale_y);
                let (x0, y0, x1, y1) = (
                    x0.round() as i64,
                    y0.round() as i64,
                    x1.round() as i64,
                    y1.round() as i64,
                );

                let dx = (x1 - x0).abs();
                let dy = -(y1 - y0).abs();

                let sx = if x0 < x1 { 1 } else { -1 };
                let sy = if y0 < y1 { 1 } else { -1 };

                let mut err = dx + dy;

                let (mut x, mut y) = (x0, y0);

                loop {
                    if x >= 0 && x < SCREEN_WIDTH as i64 && y >= 0 && y < SCREEN_HEIGHT as i64 {
                        let pixel_index = ((y as usize * SCREEN_WIDTH) + x as usize) * 4;
                        let pixel = &mut frame[pixel_index..pixel_index + 4];

                        pixel.copy_from_slice(&[0, 0, 255, 255]);
                    }

                    if x == x1 && y == y1 {
                        break;
                    }

                    if err * 2 >= dy {
                        if x == x1 {
                            break;
                        }
                        err += dy;
                        x += sx;
                    }

                    if err * 2 <= dx {
                        if y == y1 {
                            break;
                        }
                        err += dx;
                        y += sy;
                    }
                }
                pixels.render().expect("Failed to render");
            }
            _ => {}
        }
    });
}

fn world_to_screen(
    world_x: f64,
    world_y: f64,
    offset_x: f64,
    offset_y: f64,
    scale_x: f64,
    scale_y: f64,
) -> (f64, f64) {
    let screen_x = (world_x - offset_x) * scale_x;
    let screen_y = (-world_y - offset_y) * scale_y;
    (screen_x, screen_y)
}

fn screen_to_world(
    screen_x: f64,
    screen_y: f64,
    offset_x: f64,
    offset_y: f64,
    scale_x: f64,
    scale_y: f64,
) -> (f64, f64) {
    let world_x = screen_x / scale_x + offset_x;
    let world_y = -screen_y / scale_y - offset_y;
    (world_x, world_y)
}

fn clear(frame: &mut [u8]) {
    for pixel in frame.chunks_exact_mut(4) {
        pixel.copy_from_slice(&[0, 0, 0, 255]);
    }
}

fn draw_line(x0: f64, y0: f64, x1: f64, y1: f64, frame: &mut [u8]) {
    let dx = (x1 - x0).abs();
    let sx = if x0 < x1 { 1.0 } else { -1.0 };
    let dy = -(y1 - y0).abs();
    let sy = if y0 < y1 { 1.0 } else { -1.0 };
    let mut error = dx + dy;

    let width = SCREEN_WIDTH as f64;
    let height = SCREEN_HEIGHT as f64;

    let mut x = x0;
    let mut y = y0;

    loop {
        // Draw pixel
        if x >= 0.0 && x < width && y >= 0.0 && y < height {
            let pixel_index = ((y as usize * SCREEN_WIDTH) + x as usize) * 4;
            let pixel = &mut frame[pixel_index..pixel_index + 4];
            pixel.copy_from_slice(&[255, 0, 0, 255]);
        }

        // Move to next pixel
        if x == x1 && y == y1 {
            break;
        }

        if error * 2.0 >= dy {
            if x == x1 {
                break;
            }
            error += dy;
            x += sx;
        }
        if error * 2.0 <= dx {
            if y == y1 {
                break;
            }
            error += dx;
            y += sy;
        }
    }
}
