use std::sync::Mutex;

use ofm_core::advance_results::AdvanceMatchResult;
use ofm_core::state::StateManager;

pub struct SharedGameState {
    pub state: StateManager,
    pub recap_results: Mutex<Vec<AdvanceMatchResult>>,
    pub match_snapshot: Mutex<Option<engine::MatchSnapshot>>,
    pub match_mode: Mutex<String>,
    pub match_fixture_index: Mutex<usize>,
}

impl SharedGameState {
    pub fn new() -> std::sync::Arc<Self> {
        std::sync::Arc::new(Self {
            state: StateManager::new(),
            recap_results: Mutex::new(vec![]),
            match_snapshot: Mutex::new(None),
            match_mode: Mutex::new(String::new()),
            match_fixture_index: Mutex::new(0),
        })
    }
}
