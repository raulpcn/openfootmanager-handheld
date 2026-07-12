use std::sync::{Arc, Mutex};

use ofm_core::game::Game;

pub struct SharedGameState {
    pub game: Mutex<Option<Game>>,
}

impl SharedGameState {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            game: Mutex::new(None),
        })
    }
}
