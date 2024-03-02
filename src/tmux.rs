use std::{env, process::Command};

pub enum Direction {
    Left,
    Down,
    Up,
    Right,
}

impl Direction {
    fn serialize(&self) -> &str {
        match self {
            Self::Left => "{left-of}",
            Self::Down => "{down-of}",
            Self::Up => "{up-of}",
            Self::Right => "{right-of}",
        }
    }
}

pub fn select_tmux_panel(direction: Direction) {
    let Ok(tmux) = env::var("TMUX") else { return };
    let Some(socket) = tmux.split(',').next() else {
        return;
    };
    let Ok(_) = Command::new("tmux")
        .args(["-S", socket])
        .args(["select-pane", "-t", direction.serialize()])
        .output()
    else {
        return;
    };
}
