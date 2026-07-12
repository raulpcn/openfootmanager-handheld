mod font;
mod main_menu;
mod screen;

use minifb::{Key, KeyRepeat, Window, WindowOptions};
use screen::ScreenManager;

const W: usize = 640;
const H: usize = 480;

fn main() {
    let mut window = Window::new(
        "OpenFootManager Handheld",
        W,
        H,
        WindowOptions::default(),
    )
    .expect("Unable to open window");

    let mut manager = ScreenManager::new(vec![Box::new(main_menu::MainMenu::new())]);
    let mut buf = vec![0u32; W * H];

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let keys = window.get_keys_pressed(KeyRepeat::No);
        if !keys.is_empty() {
            manager.handle_input(&keys);
        }
        manager.render(&mut buf);
        window.update_with_buffer(&buf, W, H).unwrap();
    }
}
