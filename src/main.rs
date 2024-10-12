use std::f32::consts::PI;
use pixels::{Pixels, SurfaceTexture};
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};


#[derive(Default)]
struct App {
    window: Option<Window>,
    pixels: Option<Pixels>,
    angle: f32, // Store the rotation angle here.
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = event_loop.create_window(Window::default_attributes()).unwrap();
        self.window = Some(window);

        let surface_texture = SurfaceTexture::new(800, 600, self.window.as_ref().unwrap());
        self.pixels = Some(Pixels::new(800, 600, surface_texture).unwrap());

        println!("Window has resumed and pixel buffer initialized.");
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                println!("Close button pressed; exiting.");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                println!("Redraw requested.");

                self.angle += 0.05; // Increment the angle for smooth animation.
                if self.angle >= 2.0 * PI {
                    self.angle -= 2.0 * PI; // Keep angle within 0 to 2π.
                }

                self.draw_rotating_polygon();

                self.pixels.as_mut().unwrap().render().unwrap();

                self.window.as_ref().unwrap().request_redraw();
            }
            _ => (),
        }
    }
}

impl App {

    fn update_and_render(&mut self) {
        // Step 1: Increment the angle for rotation.
        self.angle += 0.05;
        if self.angle >= 2.0 * PI {
            self.angle -= 2.0 * PI; // Keep angle within 0 to 2π.
        }

        // Step 2: Draw the rotating polygon.
        self.draw_rotating_polygon();

        // Step 3: Render the updated pixel buffer.
        self.pixels.as_mut().unwrap().render().unwrap();

        // Step 4: Request another redraw to continue animation.
        self.window.as_ref().unwrap().request_redraw();
    }
    fn draw_rotating_polygon(&mut self) {
        let frame = self.pixels.as_mut().unwrap().frame_mut();
        frame.fill(0x00); // Clear the frame.

        let (cx, cy) = (400.0, 300.0); // Center of the window.

        // Define a hexagon with six vertices around the origin (0, 0).
        let radius = 100.0;
        let hexagon_points: Vec<(f32, f32)> = (0..6)
            .map(|i| {
                let theta = i as f32 * PI / 3.0; // 60-degree increments.
                (radius * theta.cos(), radius * theta.sin())
            })
            .collect();

        // Rotate and translate each point to the window center.
        let rotated_points: Vec<(usize, usize)> = hexagon_points
            .iter()
            .map(|(x, y)| rotate_point(*x, *y, self.angle, cx, cy))
            .collect();

        // Draw the polygon by connecting each pair of points.
        for i in 0..6 {
            let (x1, y1) = rotated_points[i];
            let (x2, y2) = rotated_points[(i + 1) % 6]; // Connect to the next point.
            Self::draw_line(x1, y1, x2, y2, frame);
        }
    }

    // Draw a line between two points using Bresenham's algorithm.
    fn draw_line(x1: usize, y1: usize, x2: usize, y2: usize, frame: &mut [u8]) {
        let dx = (x2 as isize - x1 as isize).abs();
        let dy = -(y2 as isize - y1 as isize).abs();
        let mut sx = if x1 < x2 { 1 } else { -1 };
        let mut sy = if y1 < y2 { 1 } else { -1 };
        let mut err = dx + dy;

        let mut x = x1 as isize;
        let mut y = y1 as isize;

        while x != x2 as isize || y != y2 as isize {
            let index = (y as usize * 800 + x as usize) * 4;
            if index < frame.len() - 4 {
                frame[index] = 0xFF;     // Red
                frame[index + 1] = 0x00; // Green
                frame[index + 2] = 0x00; // Blue
                frame[index + 3] = 0xFF; // Alpha
            }

            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x += sx;
            }
            if e2 <= dx {
                err += dx;
                y += sy;
            }
        }
    }
}

// Rotate a point around the origin, then translate it to the window center.
fn rotate_point(x: f32, y: f32, angle: f32, cx: f32, cy: f32) -> (usize, usize) {
    let cos_theta = angle.cos();
    let sin_theta = angle.sin();

    let rotated_x = cos_theta * x - sin_theta * y;
    let rotated_y = sin_theta * x + cos_theta * y;

    let final_x = rotated_x + cx;
    let final_y = rotated_y + cy;

    (final_x as usize, final_y as usize)
}

fn main() {
    let event_loop = EventLoop::new().unwrap();
    let app = Arc::new(Mutex::new(App::default())); // Wrap App in Arc<Mutex>.

    // Clone the Arc to share with the animation thread.
    let app_for_thread = Arc::clone(&app);

    // Step 1: Start the animation thread.
    thread::spawn(move || loop {
        let start = Instant::now();

        // Safely lock the app and update/render.
        if let Ok(mut app) = app_for_thread.lock() {
            app.update_and_render();
        }

        // Ensure the frame duration is 40ms.
        let elapsed = start.elapsed();
        if elapsed < Duration::from_millis(40) {
            thread::sleep(Duration::from_millis(40) - elapsed);
        }
    });

    // Step 2: Run the event loop, accessing app through Arc<Mutex>.
    event_loop.run_app(&mut *app.lock().unwrap());
}
