use minifb::Key;

pub enum ScreenAction {
    None,
    Exit,
}

pub trait Screen {
    fn handle_key(&mut self, key: Key) -> ScreenAction;
    fn render(&self, buf: &mut [u32]);
}

pub struct ScreenManager {
    screens: Vec<Box<dyn Screen>>,
    current: usize,
}

impl ScreenManager {
    pub fn new(screens: Vec<Box<dyn Screen>>) -> Self {
        Self {
            screens,
            current: 0,
        }
    }

    pub fn handle_input(&mut self, keys: &[Key]) {
        for &key in keys {
            match self.screens[self.current].handle_key(key) {
                ScreenAction::None => {}
                ScreenAction::Exit => std::process::exit(0),
            }
        }
    }

    pub fn render(&self, buf: &mut [u32]) {
        self.screens[self.current].render(buf);
    }
}
