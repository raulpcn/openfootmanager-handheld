use std::cell::RefCell;
use std::sync::{Arc, Mutex};

use chrono::Datelike;
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
                let clock = ofm_core::clock::GameClock::new(chrono::Utc::now());
                let manager = domain::manager::Manager::new(
                    "handheld_manager".into(),
                    "Manager".into(),
                    "".into(),
                    "1990-01-01".into(),
                    "ENG".into(),
                );
                let game = ofm_core::game::Game::new(
                    clock, manager, wd.teams, wd.players, wd.staff, vec![],
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

        // Set manager's team
        sw.game.manager.team_id = Some(team_id.to_string());

        // Build a minimal league from the selected team's country
        let country = sw
            .game
            .teams
            .iter()
            .find(|t| t.id == team_id)
            .map(|t| t.football_nation.clone())
            .unwrap_or_default();

        let team_ids: Vec<String> = sw
            .game
            .teams
            .iter()
            .filter(|t| t.football_nation == country)
            .map(|t| t.id.clone())
            .collect();

        if team_ids.len() >= 2 {
            let country_lower = country.to_lowercase();
            let country_label = ofm_core::nations::nation_display_name(&country);
            let def = ofm_core::generator::CompetitionDefinition {
                id: format!("{country_lower}-d1"),
                name: format!("{country_label} Division 1"),
                r#type: domain::league::CompetitionType::League,
                scope: domain::league::CompetitionScope::Domestic,
                region_id: None,
                country_id: Some(country.clone()),
                required_region_ids: vec![],
                priority: 0,
                format: ofm_core::generator::FormatDef {
                    kind: domain::league::CompetitionFormat::LeagueTable,
                    legs: None,
                    group_size: None,
                    qualifiers_per_group: None,
                    best_third_qualifiers: None,
                },
                participants: ofm_core::generator::ParticipantSpec {
                    explicit: Some(team_ids),
                    selector: None,
                },
                berths: vec![],
                season_start_month: Some(8),
                season_start_day: Some(1),
                name_key: None,
                logo: None,
            };

            let season = sw.game.clock.current_date.date_naive().year() as u32;
            if let Some(league) = ofm_core::generator::build_explicit_competition(
                &def,
                season,
                sw.game.clock.start_date,
            ) {
                sw.game.competitions.push(league);
                sw.game.sync_legacy_league();
            }
        }

        // Store the game in shared state for the dashboard
        *self.game_state.game.lock().unwrap() = Some(sw.game.clone());
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
