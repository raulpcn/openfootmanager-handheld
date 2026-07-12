use std::sync::Arc;

use minifb::Key;

use crate::font;
use crate::game_state::SharedGameState;
use crate::screen::ScreenAction;

/// Match screen — shows the live match snapshot. Reads from SharedGameState
/// so it always reflects the latest data from the backend.
pub struct MatchScreen {
    game_state: Arc<SharedGameState>,
}

impl MatchScreen {
    pub fn new(game_state: Arc<SharedGameState>) -> Self {
        Self { game_state }
    }
}

impl crate::screen::Screen for MatchScreen {
    fn handle_key(&mut self, key: Key) -> ScreenAction {
        match key {
            Key::Enter | Key::Escape => {
                // Clear the snapshot and return to dashboard
                *self.game_state.match_snapshot.lock().unwrap() = None;
                ScreenAction::SwitchTo("dashboard")
            }
            _ => ScreenAction::None,
        }
    }

    fn render(&self, buf: &mut [u32]) {
        buf.fill(0x000000);

        let snap = {
            let guard = self.game_state.match_snapshot.lock().unwrap();
            guard.clone()
        };

        let Some(snap) = snap else {
            font::draw_text(buf, "No match data", 200, 200, 0xFF4444, 2);
            font::draw_text(buf, "Press any key to continue", 120, 260, 0x666666, 1);
            return;
        };

        font::draw_text(buf, "Match Day", 220, 20, 0x00FF00, 3);

        // Home team
        font::draw_text(buf, &snap.home_team.name, 60, 80, 0xFFFFFF, 2);
        font::draw_text(buf, &snap.home_team.formation, 60, 110, 0x888888, 1);

        // Score
        let score = format!("{} - {}", snap.home_score, snap.away_score);
        font::draw_text(buf, &score, 270, 80, 0xFFFFFF, 3);

        // Away team
        font::draw_text(buf, &snap.away_team.name, 400, 80, 0xFFFFFF, 2);
        font::draw_text(buf, &snap.away_team.formation, 400, 110, 0x888888, 1);

        // Phase and minute
        let phase = format!("{:?} - {}'", snap.phase, snap.current_minute);
        font::draw_text(buf, &phase, 200, 160, 0xAAAAAA, 2);

        // Possession
        let poss = format!(
            "Possession: {}% - {}%",
            snap.home_possession_pct, snap.away_possession_pct
        );
        font::draw_text(buf, &poss, 140, 200, 0x888888, 1);

        // Recent events
        font::draw_text(buf, "Events:", 60, 260, 0x888888, 1);
        let start = snap.events.len().saturating_sub(6);
        for (i, evt) in snap.events[start..].iter().enumerate() {
            let y = 290 + i as i32 * 20;
            let line = format!("{}' {:?}", evt.minute, evt.event_type);
            font::draw_text(buf, &line, 80, y, 0xCCCCCC, 1);
        }

        font::draw_text(buf, "Press any key to continue", 120, 440, 0x666666, 1);
    }
}
