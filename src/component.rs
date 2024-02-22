use crossterm::event::{Event, KeyEvent, MouseEvent};

pub mod app;

pub trait Component: Sized {
    fn iter_children(self) -> dyn Iterator<Item = dyn Component>;

    fn handle_event(self, event: Option<Event>) -> Self {
        match event {
            Some(Event::Key(key_event)) => self.handle_key_event(key_event),
            Some(Event::Mouse(mouse_event)) => self.handle_mouse_event(mouse_event),
            _ => self,
        }
    }

    fn handle_key_event(self, _key_event: KeyEvent) -> Self {
        self
    }

    fn handle_mouse_event(self, _mouse_event: MouseEvent) -> Self {
        self
    }
}
