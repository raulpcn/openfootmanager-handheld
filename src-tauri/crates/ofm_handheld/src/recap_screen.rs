use std::sync::Arc;

use minifb::Key;

use crate::font;
use crate::game_state::SharedGameState;
use crate::screen::ScreenAction;

const VISIBLE_ROWS: usize = 8;
const ROW_HEIGHT: i32 = 22;

/// Recap screen — shows match results after advancing, matching the desktop
/// DashboardResultsRecapModal. Reads results from SharedGameState.
pub struct RecapScreen {
    game_state: Arc<SharedGameState>,
    scroll: usize,
}

impl RecapScreen {
    pub fn new(game_state: Arc<SharedGameState>) -> Self {
        Self {
            game_state,
            scroll: 0,
        }
    }
}

impl crate::screen::Screen for RecapScreen {
    fn handle_key(&mut self, key: Key) -> ScreenAction {
        match key {
            Key::Up => {
                self.scroll = self.scroll.saturating_sub(1);
                ScreenAction::None
            }
            Key::Down => {
                let len = self
                    .game_state
                    .recap_results
                    .lock()
                    .map(|r| r.len())
                    .unwrap_or(0);
                if self.scroll + VISIBLE_ROWS < len {
                    self.scroll += 1;
                }
                ScreenAction::None
            }
            Key::Enter | Key::Escape => {
                // Clear recap and return to dashboard (same as desktop: close recap modal)
                self.game_state.recap_results.lock().unwrap().clear();
                self.scroll = 0;
                ScreenAction::SwitchTo("dashboard")
            }
            _ => ScreenAction::None,
        }
    }

    fn render(&self, buf: &mut [u32]) {
        buf.fill(0x000000);

        font::draw_text(buf, "Results", 240, 20, 0x00FF00, 3);

        let date = self
            .game_state
            .state
            .get_game(|g| g.clock.current_date.format("%Y-%m-%d").to_string())
            .unwrap_or_default();
        font::draw_text(buf, &date, 260, 55, 0x888888, 1);

        let results = self.game_state.recap_results.lock().unwrap();

        if results.is_empty() {
            font::draw_text(buf, "No matches played today.", 160, 200, 0xAAAAAA, 2);
            font::draw_text(buf, "Press any key to continue", 120, 260, 0x666666, 1);
            return;
        }

        let end = (self.scroll + VISIBLE_ROWS).min(results.len());
        for (i, idx) in (self.scroll..end).enumerate() {
            let r = &results[idx];
            let y = 90 + i as i32 * ROW_HEIGHT;
            let color = if r.involves_user { 0xFFFFFF } else { 0xAAAAAA };

            let score = if let (Some(hp), Some(ap)) = (r.home_penalties, r.away_penalties) {
                format!("{} - {} (pens {}-{})", r.home_goals, r.away_goals, hp, ap)
            } else {
                format!("{} - {}", r.home_goals, r.away_goals)
            };

            let line = format!("{} {} {}", r.home_team, score, r.away_team);
            font::draw_text(buf, &line, 30, y, color, 1);

            if !r.competition.is_empty() {
                font::draw_text(buf, &r.competition, 450, y, 0x666666, 1);
            }
        }

        if self.scroll > 0 {
            font::draw_text(buf, "^", 610, 80, 0x888888, 1);
        }
        if end < results.len() {
            font::draw_text(buf, "v", 610, 90 + VISIBLE_ROWS as i32 * ROW_HEIGHT, 0x888888, 1);
        }

        drop(results);
        font::draw_text(buf, "Press any key to continue", 120, 440, 0x666666, 1);
    }
}
