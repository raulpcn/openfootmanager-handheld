use std::sync::Arc;

use minifb::Key;

use crate::font;
use crate::game_state::SharedGameState;
use crate::screen::ScreenAction;

const MENU_ITEMS: &[&str] = &["Continue", "Save", "Exit"];

pub struct Dashboard {
    game_state: Arc<SharedGameState>,
    selected: usize,
}

impl Dashboard {
    pub fn new(game_state: Arc<SharedGameState>) -> Self {
        Self {
            game_state,
            selected: 0,
        }
    }

    fn with_game<R>(&self, f: impl FnOnce(&ofm_core::game::Game) -> R) -> Option<R> {
        let guard = self.game_state.game.lock().ok()?;
        guard.as_ref().map(f)
    }
}

impl crate::screen::Screen for Dashboard {
    fn handle_key(&mut self, key: Key) -> ScreenAction {
        match key {
            Key::Up => {
                self.selected = self.selected.saturating_sub(1);
            }
            Key::Down => {
                if self.selected + 1 < MENU_ITEMS.len() {
                    self.selected += 1;
                }
            }
            Key::Enter => {
                match MENU_ITEMS[self.selected] {
                    "Exit" => return ScreenAction::Exit,
                    _ => {}
                }
            }
            Key::Escape => return ScreenAction::Exit,
            _ => {}
        }
        ScreenAction::None
    }

    fn render(&self, buf: &mut [u32]) {
        buf.fill(0x000000);

        font::draw_text(buf, "OpenFootManager Handheld", 140, 20, 0x00FF00, 2);

        let (manager, club, league, date) = self
            .with_game(|g| {
                let manager = format!(
                    "{} {}",
                    g.manager.first_name, g.manager.last_name
                );
                let club = g
                    .manager
                    .team_id
                    .as_ref()
                    .and_then(|id| g.teams.iter().find(|t| &t.id == id))
                    .map(|t| t.name.clone())
                    .unwrap_or_else(|| "Unassigned".into());
                let league = g
                    .league
                    .as_ref()
                    .map(|l| l.name.clone())
                    .unwrap_or_else(|| "N/A".into());
                let date = g.clock.current_date.format("%Y-%m-%d").to_string();
                (manager, club, league, date)
            })
            .unwrap_or_else(|| {
                (
                    "N/A".into(),
                    "N/A".into(),
                    "N/A".into(),
                    "N/A".into(),
                )
            });

        let y_start = 80;
        let items = [
            ("Manager:", &manager),
            ("Club:", &club),
            ("League:", &league),
            ("Date:", &date),
        ];

        for (i, (label, value)) in items.iter().enumerate() {
            let y = y_start + i as i32 * 30;
            font::draw_text(buf, label, 60, y, 0x888888, 2);
            font::draw_text(buf, value, 220, y, 0xFFFFFF, 2);
        }

        let menu_y = 260;
        font::draw_text(buf, "---", 60, menu_y - 10, 0x444444, 1);

        for (i, item) in MENU_ITEMS.iter().enumerate() {
            let y = menu_y + i as i32 * 35;
            let is_selected = i == self.selected;

            if is_selected {
                font::fill_rect(buf, 50, y - 4, 200, 28, 0x222244);
                font::draw_text(buf, "> ", 60, y, 0xFFFFFF, 2);
                font::draw_text(buf, item, 90, y, 0xFFFFFF, 2);
            } else {
                font::draw_text(buf, item, 90, y, 0xAAAAAA, 2);
            }
        }
    }
}
