use std::sync::Arc;

use minifb::Key;

use crate::font;
use crate::game_state::SharedGameState;
use crate::screen::ScreenAction;

const MENU_ITEMS: &[&str] = &["Continue", "Save", "Exit"];

pub struct Dashboard {
    game_state: Arc<SharedGameState>,
    selected: usize,
    /// Blockers to display if present
    blockers: Vec<serde_json::Value>,
}

impl Dashboard {
    pub fn new(game_state: Arc<SharedGameState>) -> Self {
        Self {
            game_state,
            selected: 0,
            blockers: vec![],
        }
    }

    fn with_game<R>(&self, f: impl FnOnce(&ofm_core::game::Game) -> R) -> Option<R> {
        self.game_state.state.get_game(|g| f(g))
    }

    fn do_continue(&mut self) -> ScreenAction {
        self.blockers.clear();

        // Check blockers (same path as desktop: useAdvanceTime → checkBlockingActions)
        let blockers = self.game_state.state.get_game(|g| {
            ofm_app::time_blockers::compute_blocking_actions(g)
        });
        if let Some(blockers) = blockers {
            if !blockers.is_empty() {
                self.blockers = blockers;
                return ScreenAction::None;
            }
        }

        // Advance time (same path as desktop: advanceTimeWithMode → "delegate")
        let result = ofm_app::time_advancement::advance_time_with_mode(
            &self.game_state.state,
            "delegate",
        );

        match result {
            Ok(resp) => match resp.action.as_str() {
                "advanced" => {
                    if let Some(game) = resp.game {
                        self.game_state.state.set_game(game);
                    }
                    // Store results for the recap screen (same data desktop shows)
                    *self.game_state.recap_results.lock().unwrap() = resp.results;
                    ScreenAction::SwitchTo("recap")
                }
                "live_match" => {
                    // Store snapshot for the match screen (same as desktop navigate("/match"))
                    if let Some(snap) = resp.snapshot {
                        *self.game_state.match_snapshot.lock().unwrap() = Some(snap);
                        *self.game_state.match_mode.lock().unwrap() =
                            resp.mode.unwrap_or_else(|| "delegate".into());
                        *self.game_state.match_fixture_index.lock().unwrap() =
                            resp.fixture_index.unwrap_or(0);
                        return ScreenAction::SwitchTo("match");
                    }
                    ScreenAction::None
                }
                "fired" => {
                    if let Some(game) = resp.game {
                        self.game_state.state.set_game(game);
                    }
                    ScreenAction::SwitchTo("fired")
                }
                _ => ScreenAction::None,
            },
            Err(_) => ScreenAction::None,
        }
    }
}

impl crate::screen::Screen for Dashboard {
    fn handle_key(&mut self, key: Key) -> ScreenAction {
        // Dismiss blockers with any key
        if !self.blockers.is_empty() {
            self.blockers.clear();
            return ScreenAction::None;
        }

        match key {
            Key::Up => {
                self.selected = self.selected.saturating_sub(1);
                ScreenAction::None
            }
            Key::Down => {
                if self.selected + 1 < MENU_ITEMS.len() {
                    self.selected += 1;
                }
                ScreenAction::None
            }
            Key::Enter => match MENU_ITEMS[self.selected] {
                "Continue" => return self.do_continue(),
                "Exit" => return ScreenAction::Exit,
                _ => ScreenAction::None,
            },
            Key::Escape => ScreenAction::Exit,
            _ => ScreenAction::None,
        }
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
                ("N/A".into(), "N/A".into(), "N/A".into(), "N/A".into())
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

        // Show blockers
        if !self.blockers.is_empty() {
            let by = 220;
            font::draw_text(buf, "Attention required:", 60, by, 0xFF8800, 2);
            for (i, blocker) in self.blockers.iter().take(4).enumerate() {
                let text = blocker
                    .get("text_key")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown issue");
                let y = by + 30 + i as i32 * 24;
                font::draw_text(buf, text, 80, y, 0xCCCCCC, 1);
            }
            font::draw_text(buf, "Press any key to continue", 120, by + 140, 0x666666, 1);
            return;
        }

        // Menu
        let menu_y = 260;
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
