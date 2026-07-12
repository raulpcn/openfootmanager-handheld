use minifb::{Key, Window, WindowOptions};

fn main() {
    let mut window = Window::new(
        "OpenFootManager Handheld",
        640,
        480,
        WindowOptions::default(),
    )
    .expect("Unable to open window");

    let black = vec![0u32; 640 * 480];

    while window.is_open() && !window.is_key_down(Key::Escape) {
        window.update_with_buffer(&black, 640, 480).unwrap();
    }
}
