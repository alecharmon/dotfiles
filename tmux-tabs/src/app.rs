//! Sidebar application state and the pure event→action reducer.
//!
//! `handle_event` is the single place where mouse clicks, wheel scrolling and
//! key presses turn into state changes and external `Action`s. It takes only
//! plain data, so integration tests can drive it directly — no terminal, no
//! tmux — and assert on the resulting actions and scroll/menu state.

use crate::layout::{build_lines, Target, VisualLine};

/// An abstract input event, decoupled from crossterm so tests can synthesise it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    /// Left-button press at 0-based row within the sidebar pane.
    Click {
        row: u16,
    },
    /// Second left-button press on the same row within the double-click window.
    DoubleClick {
        row: u16,
    },
    /// Right or middle button press at 0-based row.
    RightClick {
        row: u16,
    },
    ScrollUp,
    ScrollDown,
    Key(char),
}

/// A side effect for the runtime to carry out (tmux calls / process exit).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    SwitchTo(String),
    Rename(String),
    KillWindow(String),
    /// Kill the window, then `wt remove` its worktree.
    KillWindowAndWorktree(String),
    ClearReady(String),
    OpenPr(String),
    RefreshPr(String),
    Quit,
}

#[derive(Debug, Clone, Default)]
pub struct State {
    pub windows: Vec<crate::model::Window>,
    pub menu_window: String,
    pub width: usize,
    pub height: usize,
    pub scroll: usize,
    pub current_window: String,
}

impl State {
    pub fn lines(&self) -> Vec<VisualLine> {
        build_lines(&self.windows, &self.menu_window, self.width)
    }

    /// Largest valid scroll offset given current content and viewport height.
    pub fn max_scroll(&self) -> usize {
        self.lines().len().saturating_sub(self.height)
    }

    fn clamp_scroll(&mut self) {
        let max = self.max_scroll();
        if self.scroll > max {
            self.scroll = max;
        }
    }

    /// Keep state consistent after the window list or viewport changes.
    pub fn on_data_changed(&mut self) {
        if !self.menu_window.is_empty() && !self.windows.iter().any(|w| w.index == self.menu_window)
        {
            self.menu_window.clear();
        }
        self.clamp_scroll();
    }

    fn open_action_menu(&mut self, index: &str) {
        if self.menu_window == index {
            self.menu_window.clear();
            return;
        }
        self.menu_window = index.to_string();
    }

    fn run_window_action(&mut self, index: &str, action: &str, out: &mut Vec<Action>) {
        match action {
            "open PR" => {
                out.push(Action::OpenPr(index.to_string()));
                self.menu_window.clear();
            }
            "refresh PR" => {
                out.push(Action::RefreshPr(index.to_string()));
                self.menu_window.clear();
            }
            "rename" => {
                out.push(Action::Rename(index.to_string()));
                self.menu_window.clear();
            }
            "kill" => {
                out.push(Action::KillWindow(index.to_string()));
                self.menu_window.clear();
            }
            "kill + rm worktree" => {
                out.push(Action::KillWindowAndWorktree(index.to_string()));
                self.menu_window.clear();
            }
            "clear ready" => {
                out.push(Action::ClearReady(index.to_string()));
                self.menu_window.clear();
            }
            _ => {}
        }
    }
}

/// Apply an event to the state, returning external actions to perform.
pub fn handle_event(state: &mut State, event: Event) -> Vec<Action> {
    let mut out = Vec::new();
    match event {
        Event::Key('q') => out.push(Action::Quit),
        Event::Key(_) => {}
        Event::ScrollUp => {
            state.scroll = state.scroll.saturating_sub(3);
        }
        Event::ScrollDown => {
            let max = state.max_scroll();
            state.scroll = (state.scroll + 3).min(max);
        }
        Event::Click { row } => {
            let idx = state.scroll + row as usize;
            let lines = state.lines();
            match lines.get(idx).and_then(|l| l.target.clone()) {
                Some(Target::SwitchTo(index)) | Some(Target::OpenPr(index)) => {
                    state.current_window = index.clone();
                    out.push(Action::SwitchTo(index));
                }
                Some(Target::RunAction { index, action }) => {
                    state.run_window_action(&index, &action, &mut out);
                }
                None => {}
            }
        }
        Event::DoubleClick { row } => {
            let idx = state.scroll + row as usize;
            if let Some(Target::OpenPr(index)) = state.lines().get(idx).and_then(|l| l.target.clone())
            {
                out.push(Action::OpenPr(index));
            }
        }
        Event::RightClick { row } => {
            let idx = state.scroll + row as usize;
            let lines = state.lines();
            match lines.get(idx).and_then(|l| l.target.clone()) {
                Some(Target::SwitchTo(index)) | Some(Target::OpenPr(index)) => {
                    state.open_action_menu(&index);
                }
                _ => {}
            }
        }
    }
    state.clamp_scroll();
    out
}

/// Convenience used by both runtime and tests to make a `VisualLine` viewport.
pub fn visible<'a>(lines: &'a [VisualLine], scroll: usize, height: usize) -> &'a [VisualLine] {
    let start = scroll.min(lines.len());
    let end = (scroll + height).min(lines.len());
    &lines[start..end]
}
