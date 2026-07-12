use minifb::Key;

pub enum ScreenAction {
    None,
    Exit,
    SwitchTo(&'static str),
}

pub trait Screen {
    fn handle_key(&mut self, key: Key) -> ScreenAction;
    fn render(&self, buf: &mut [u32]);
}

pub struct ScreenManager {
    screens: std::collections::HashMap<&'static str, Box<dyn Screen>>,
    current: &'static str,
}

impl ScreenManager {
    pub fn new(screens: Vec<(&'static str, Box<dyn Screen>)>) -> Self {
        let current = screens[0].0;
        let map = screens.into_iter().collect();
        Self {
            screens: map,
            current,
        }
    }

    pub fn handle_input(&mut self, keys: &[Key]) {
        for &key in keys {
            let action = self.screens.get_mut(self.current).unwrap().handle_key(key);
            match action {
                ScreenAction::None => {}
                ScreenAction::Exit => std::process::exit(0),
                ScreenAction::SwitchTo(name) => {
                    if self.screens.contains_key(name) {
                        self.current = name;
                    }
                }
            }
        }
    }

    pub fn render(&self, buf: &mut [u32]) {
        self.screens[self.current].render(buf);
    }
}
