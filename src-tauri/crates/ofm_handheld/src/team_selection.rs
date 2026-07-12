use std::cell::RefCell;
use std::sync::{Arc, Mutex};

use minifb::Key;

use crate::font;
use crate::game_state::SharedGameState;
use crate::screen::ScreenAction;

const VISIBLE_ROWS: usize = 8;
const ROW_HEIGHT: i32 = 28;
const TOP_Y: i32 = 80;

#[derive(Clone)]
struct TeamEntry {
    id: String,
    name: String,
    country: String,
    reputation: u32,
}

struct SharedWorld {
    teams: Vec<TeamEntry>,
    game: ofm_core::game::Game,
}

pub struct TeamSelection {
    world: Arc<Mutex<Option<SharedWorld>>>,
    game_state: Arc<SharedGameState>,
    teams: RefCell<Vec<TeamEntry>>,
    scroll: usize,
    selected: usize,
}

impl TeamSelection {
    pub fn new(game_state: Arc<SharedGameState>) -> Self {
        let world = Arc::new(Mutex::new(None::<SharedWorld>));
        let world_clone = world.clone();

        std::thread::spawn(move || {
            let result = std::panic::catch_unwind(|| {
                eprintln!("[handheld] starting world generation...");
                let wd = ofm_core::generator::generate_world_data(None);
                eprintln!("[handheld] world generated, {} teams", wd.teams.len());

                let startup_options = ofm_app::game_setup::StartupOptions {
                    start_year: ofm_app::game_setup::default_start_year(),
                    start_phase: ofm_app::game_setup::StartPhase::SeasonStart,
                    history_depth_years: ofm_app::game_setup::DEFAULT_GENERATED_HISTORY_DEPTH_YEARS,
                };
                let clock = ofm_app::game_setup::game_clock_for_world(
                    &startup_options,
                    &wd.metadata,
                )
                .expect("failed to create game clock");
                let manager = domain::manager::Manager::new(
                    "handheld_manager".into(),
                    "Manager".into(),
                    "".into(),
                    "1990-01-01".into(),
                    "ENG".into(),
                );

                let (game, _stats) = ofm_app::game_setup::build_game_from_world_data(
                    clock,
                    manager,
                    &startup_options,
                    wd,
                );

                let mut teams: Vec<TeamEntry> = game
                    .teams
                    .iter()
                    .map(|t| TeamEntry {
                        id: t.id.clone(),
                        name: t.name.clone(),
                        country: t.country.clone(),
                        reputation: t.reputation,
                    })
                    .collect();
                teams.sort_by(|a, b| a.country.cmp(&b.country).then(a.name.cmp(&b.name)));
                eprintln!("[handheld] teams sorted, {} entries", teams.len());

                Some(SharedWorld { teams, game })
            });

            let outcome = match result {
                Ok(opt) => opt,
                Err(panic) => {
                    let msg = if let Some(s) = panic.downcast_ref::<&str>() {
                        s.to_string()
                    } else if let Some(s) = panic.downcast_ref::<String>() {
                        s.clone()
                    } else {
                        "unknown panic".into()
                    };
                    eprintln!("[handheld] world generation FAILED: {msg}");
                    None
                }
            };

            *world_clone.lock().unwrap() = outcome;
        });

        Self {
            world,
            game_state,
            teams: RefCell::new(vec![]),
            scroll: 0,
            selected: 0,
        }
    }

    fn ensure_teams_loaded(&self) {
        if !self.teams.borrow().is_empty() {
            return;
        }
        let guard = self.world.lock().unwrap();
        if let Some(ref sw) = *guard {
            *self.teams.borrow_mut() = sw.teams.clone();
        }
    }

    fn select_team(&self, team_id: &str) {
        let mut guard = self.world.lock().unwrap();
        let sw = match guard.as_mut() {
            Some(sw) => sw,
            None => return,
        };

        // Hemisphere fix: align clock to the club's actual season-start date
        // so southern-hemisphere clubs begin at the right time of year.
        // Same logic as desktop select_team.
        if ofm_app::game_setup::start_phase_for_game(&sw.game)
            == ofm_app::game_setup::StartPhase::SeasonStart
        {
            if let Some(actual_start) =
                ofm_app::game_setup::team_season_anchor(&sw.game, team_id)
            {
                if actual_start < sw.game.clock.current_date {
                    sw.game.clock.current_date = actual_start;
                    sw.game.clock.start_date = actual_start;
                    ofm_app::game_setup::rebuild_competitions_for_management_date(
                        &mut sw.game,
                        actual_start,
                    );
                    sw.game.national_teams.clear();
                    ofm_app::game_setup::ensure_multi_competition_foundations(&mut sw.game);
                }
            }
        }

        // Scope active regions/competitions to the user's team — same as desktop.
        let (resolved_regions, resolved_competitions) =
            ofm_app::game_setup::resolve_simulation_scope(
                &sw.game,
                team_id,
                None,
                None,
            )
            .unwrap_or_else(|_| (vec![], vec![]));
        sw.game.active_region_ids = resolved_regions;
        sw.game.active_competition_ids = resolved_competitions;

        let start_phase = ofm_app::game_setup::start_phase_for_game(&sw.game);
        let result = ofm_app::game_setup::bootstrap_team_selection(
            &mut sw.game,
            team_id,
            start_phase,
            domain::stats::StatsState::default(),
        );
        if let Err(e) = result {
            eprintln!("[handheld] bootstrap_team_selection failed: {e}");
            return;
        }

        ofm_core::player_identity::upgrade_game_player_identities(&mut sw.game);

        self.game_state.state.set_game(sw.game.clone());
    }
}

impl crate::screen::Screen for TeamSelection {
    fn handle_key(&mut self, key: Key) -> ScreenAction {
        self.ensure_teams_loaded();
        if self.teams.borrow().is_empty() {
            return ScreenAction::None;
        }
        match key {
            Key::Up => {
                self.selected = self.selected.saturating_sub(1);
                if self.selected < self.scroll {
                    self.scroll = self.selected;
                }
            }
            Key::Down => {
                if self.selected + 1 < self.teams.borrow().len() {
                    self.selected += 1;
                }
                if self.selected >= self.scroll + VISIBLE_ROWS {
                    self.scroll = self.selected - VISIBLE_ROWS + 1;
                }
            }
            Key::Enter => {
                let team_id = self.teams.borrow()[self.selected].id.clone();
                self.select_team(&team_id);
                return ScreenAction::SwitchTo("dashboard");
            }
            Key::Escape => return ScreenAction::SwitchTo("main_menu"),
            _ => {}
        }
        ScreenAction::None
    }

    fn render(&self, buf: &mut [u32]) {
        self.ensure_teams_loaded();
        buf.fill(0x000000);

        let teams = self.teams.borrow();
        if teams.is_empty() {
            font::draw_text(buf, "Generating world...", 180, 200, 0xAAAAAA, 2);
            font::draw_text(buf, "Please wait", 220, 240, 0x666666, 2);
            return;
        }

        font::draw_text(buf, "Select a Team", 200, 20, 0x00FF00, 3);
        font::draw_text(
            buf,
            &format!("{}/{}", self.selected + 1, teams.len()),
            480, 26, 0x888888, 1,
        );

        let end = (self.scroll + VISIBLE_ROWS).min(teams.len());
        for (i, idx) in (self.scroll..end).enumerate() {
            let team = &teams[idx];
            let y = TOP_Y + i as i32 * ROW_HEIGHT;
            let is_selected = idx == self.selected;

            if is_selected {
                font::fill_rect(buf, 20, y - 2, 600, ROW_HEIGHT - 4, 0x222244);
            }

            let rep_label = match team.reputation {
                750.. => "WC",
                600.. => "ST",
                400.. => "AV",
                _ => "DV",
            };
            let color = if is_selected { 0xFFFFFF } else { 0xAAAAAA };

            let line = format!("[{}] {} - {}", rep_label, team.name, team.country);
            font::draw_text(buf, &line, 30, y, color, 1);
        }

        if self.scroll > 0 {
            font::draw_text(buf, "^", 620, TOP_Y - 10, 0x888888, 1);
        }
        if end < teams.len() {
            font::draw_text(buf, "v", 620, TOP_Y + VISIBLE_ROWS as i32 * ROW_HEIGHT, 0x888888, 1);
        }
    }
}
