use minifb::Key;

use crate::font;
use crate::screen::ScreenAction;

pub struct MainMenu {
    items: Vec<&'static str>,
    selected: usize,
}

impl MainMenu {
    pub fn new() -> Self {
        Self {
            items: vec!["New Game", "Load Game", "Exit"],
            selected: 0,
        }
    }
}

impl crate::screen::Screen for MainMenu {
    fn handle_key(&mut self, key: Key) -> ScreenAction {
        match key {
            Key::Up => {
                self.selected = self.selected.saturating_sub(1);
                ScreenAction::None
            }
            Key::Down => {
                if self.selected < self.items.len() - 1 {
                    self.selected += 1;
                }
                ScreenAction::None
            }
            Key::Enter => {
                println!("Selected: {}", self.items[self.selected]);
                match self.items[self.selected] {
                    "Exit" => ScreenAction::Exit,
                    _ => ScreenAction::None,
                }
            }
            _ => ScreenAction::None,
        }
    }

    fn render(&self, buf: &mut [u32]) {
        buf.fill(0x000000);

        font::draw_text(buf, "OpenFootManager Handheld", 128, 40, 0x00FF00, 3);

        for (i, item) in self.items.iter().enumerate() {
            let y = 160 + (i as i32) * 50;

            if i == self.selected {
                font::fill_rect(buf, 170, y - 4, 300, 30, 0x222244);
                font::draw_text(buf, "> ", 180, y, 0xFFFFFF, 2);
                font::draw_text(buf, item, 210, y, 0xFFFFFF, 2);
            } else {
                font::draw_text(buf, item, 210, y, 0xAAAAAA, 2);
            }
        }
    }
}
