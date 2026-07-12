mod dashboard;
mod fired_screen;
mod font;
mod game_state;
mod main_menu;
mod match_screen;
mod recap_screen;
mod screen;
mod team_selection;

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

    let game_state = game_state::SharedGameState::new();

    let mut manager = ScreenManager::new(vec![
        ("main_menu", Box::new(main_menu::MainMenu::new())),
        (
            "team_selection",
            Box::new(team_selection::TeamSelection::new(game_state.clone())),
        ),
        ("dashboard", Box::new(dashboard::Dashboard::new(game_state.clone()))),
        (
            "recap",
            Box::new(recap_screen::RecapScreen::new(game_state.clone())),
        ),
        ("match", Box::new(match_screen::MatchScreen::new(game_state.clone()))),
        ("fired", Box::new(fired_screen::FiredScreen::new(game_state))),
    ]);
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
