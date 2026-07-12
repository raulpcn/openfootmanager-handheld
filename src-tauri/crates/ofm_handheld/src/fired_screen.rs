use std::sync::Arc;

use minifb::Key;

use crate::font;
use crate::game_state::SharedGameState;
use crate::screen::ScreenAction;

/// Fired screen — shown when the manager is dismissed, matching the desktop
/// FiredModal behavior.
pub struct FiredScreen {
    game_state: Arc<SharedGameState>,
}

impl FiredScreen {
    pub fn new(game_state: Arc<SharedGameState>) -> Self {
        Self { game_state }
    }
}

impl crate::screen::Screen for FiredScreen {
    fn handle_key(&mut self, _key: Key) -> ScreenAction {
        // Any key returns to dashboard (same as desktop: FiredModal onClose)
        ScreenAction::SwitchTo("dashboard")
    }

    fn render(&self, buf: &mut [u32]) {
        buf.fill(0x000000);

        font::draw_text(buf, "Manager Sacked", 180, 120, 0xFF4444, 3);

        let manager = self
            .game_state
            .state
            .get_game(|g| format!("{} {}", g.manager.first_name, g.manager.last_name))
            .unwrap_or_else(|| "Manager".into());

        let team = self
            .game_state
            .state
            .get_game(|g| {
                g.manager
                    .team_id
                    .as_ref()
                    .and_then(|id| g.teams.iter().find(|t| &t.id == id))
                    .map(|t| t.name.clone())
                    .unwrap_or_else(|| "your club".into())
            })
            .unwrap_or_else(|| "your club".into());

        font::draw_text(buf, &format!("{} has been", manager), 160, 190, 0xFFFFFF, 2);
        font::draw_text(buf, &format!("sacked by {}.", team), 180, 220, 0xFFFFFF, 2);

        font::draw_text(buf, "Press any key to continue", 120, 340, 0x666666, 1);
    }
}
