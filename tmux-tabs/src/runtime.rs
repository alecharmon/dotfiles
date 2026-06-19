//! Terminal runtime, built on the `ratatui` TUI crate (with its re-exported
//! crossterm backend). ratatui handles double-buffered, flicker-free drawing;
//! this module wires real mouse/key events into the pure `Event`s handled by
//! `app`, and renders the visual lines produced by `layout`.

use std::io::{self, Stdout};
use std::path::PathBuf;
use std::process::Command;
use std::time::{Duration, Instant};

use ratatui::backend::CrosstermBackend;
use ratatui::crossterm::event::{
    self, DisableMouseCapture, EnableMouseCapture, Event as CtEvent, KeyCode, MouseButton,
    MouseEventKind,
};
use ratatui::crossterm::execute;
use ratatui::crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::{Terminal, TerminalOptions, Viewport};
use unicode_width::UnicodeWidthStr;

use crate::app::{handle_event, Action, Event, State};
use crate::control::move_sidebar_to_window;
use crate::layout::LineStyle;
use crate::theme::{load_theme, Theme};
use crate::tmux::{
    active_pane, clear_ready_panes, display, pane_var, tmux_windows, Ctx, RealTmux, Tmux,
};

const TICK: Duration = Duration::from_millis(200);
const REFRESH: Duration = Duration::from_secs(1);

type Term = Terminal<CrosstermBackend<Stdout>>;

pub struct Runtime {
    t: RealTmux,
    ctx: Ctx,
    state: State,
    theme: Theme,
    pane_id: String,
}

impl Runtime {
    pub fn new(ctx: Ctx) -> io::Result<Self> {
        let t = RealTmux;
        let pane_id = std::env::var("TMUX_PANE").unwrap_or_default();
        let theme = load_theme(&t);
        let mut state = State::default();
        state.current_window = pane_var(&t, &pane_id, "#{window_index}");
        Ok(Runtime {
            t,
            ctx,
            state,
            theme,
            pane_id,
        })
    }

    fn refresh_size(&mut self, term: &Term) {
        if let Ok(area) = term.size() {
            self.state.width = area.width as usize;
            self.state.height = area.height as usize;
        }
    }

    fn refresh_windows(&mut self, term: &Term) {
        self.refresh_size(term);
        self.state.windows = tmux_windows(&self.t, &self.ctx);
        self.state.on_data_changed();
    }

    fn window_panes(&self, index: &str) -> Vec<String> {
        self.state
            .windows
            .iter()
            .find(|w| w.index == index)
            .map(|w| w.panes.clone())
            .unwrap_or_default()
    }

    fn window_path(&self, index: &str) -> String {
        self.state
            .windows
            .iter()
            .find(|w| w.index == index)
            .map(|w| w.path.clone())
            .unwrap_or_default()
    }

    fn window_pr_url(&self, index: &str) -> String {
        self.state
            .windows
            .iter()
            .find(|w| w.index == index)
            .and_then(|w| w.pr.as_ref())
            .map(|pr| pr.url.clone())
            .unwrap_or_default()
    }

    fn pr_status_command(&self) -> PathBuf {
        let repo_script = PathBuf::from(&self.ctx.home).join("dev/dotfiles/scripts/pr-status");
        if repo_script.exists() {
            repo_script
        } else {
            PathBuf::from("pr-status")
        }
    }

    fn open_pr_url(&self, url: &str) {
        if url.is_empty() {
            return;
        }
        let opener = if cfg!(target_os = "macos") {
            "open"
        } else {
            "xdg-open"
        };
        let _ = Command::new(opener).arg(url).spawn();
    }

    fn refresh_pr_status(&self, index: &str) {
        let pane = active_pane(&self.t, index, &self.ctx.home);
        let path = self.window_path(index);
        if pane.is_empty() || path.is_empty() {
            return;
        }
        let _ = Command::new(self.pr_status_command())
            .args(["refresh", "--pane", &pane, "--path", &path])
            .status();
    }

    fn perform(&mut self, action: Action, term: &Term) -> bool {
        match action {
            Action::Quit => return true,
            Action::SwitchTo(index) => {
                let actual = display(&self.t, "#{window_index}");
                if index != actual {
                    move_sidebar_to_window(
                        &self.t,
                        &self.ctx,
                        &self.pane_id,
                        &index,
                        &self.state.windows,
                    );
                }
                clear_ready_panes(&self.ctx, &self.window_panes(&index));
                self.state.current_window = index;
                self.refresh_windows(term);
            }
            Action::Rename(index) => {
                let current = self
                    .state
                    .windows
                    .iter()
                    .find(|w| w.index == index)
                    .map(|w| w.name.clone())
                    .unwrap_or_default();
                let cmd = format!("rename-window -t {index} '%%'");
                let _ = self.t.run(&[
                    "command-prompt",
                    "-I",
                    &current,
                    "-p",
                    "Rename window:",
                    &cmd,
                ]);
            }
            Action::KillWindow(index) => {
                let _ = self.t.run(&["kill-window", "-t", &index]);
                self.refresh_windows(term);
            }
            Action::ClearReady(index) => {
                clear_ready_panes(&self.ctx, &self.window_panes(&index));
                self.refresh_windows(term);
            }
            Action::OpenPr(index) => {
                let url = self.window_pr_url(&index);
                self.open_pr_url(&url);
            }
            Action::RefreshPr(index) => {
                self.refresh_pr_status(&index);
                self.refresh_windows(term);
            }
        }
        false
    }

    /// Follow focus: if the active window drifted away, move the sidebar to it.
    fn follow_focus(&mut self, term: &Term) {
        let active = display(&self.t, "#{window_index}");
        if !active.is_empty() && active != self.state.current_window {
            move_sidebar_to_window(
                &self.t,
                &self.ctx,
                &self.pane_id,
                &active,
                &self.state.windows,
            );
            self.state.current_window = active;
            self.refresh_windows(term);
        }
    }

    pub fn run(&mut self) -> io::Result<()> {
        if !self.pane_id.is_empty() {
            let _ = self
                .t
                .run(&["select-pane", "-t", &self.pane_id, "-T", "tmux-tabs"]);
        }
        crate::descriptions::spawn_refresher(self.ctx.clone());

        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut term = Terminal::with_options(
            backend,
            TerminalOptions {
                viewport: Viewport::Fullscreen,
            },
        )?;
        term.hide_cursor()?;

        self.refresh_windows(&term);
        let result = self.event_loop(&mut term);

        disable_raw_mode()?;
        execute!(
            term.backend_mut(),
            DisableMouseCapture,
            LeaveAlternateScreen
        )?;
        term.show_cursor()?;
        result
    }

    fn event_loop(&mut self, term: &mut Term) -> io::Result<()> {
        self.draw(term)?;
        let mut last_refresh = Instant::now();
        loop {
            if event::poll(TICK)? {
                if let Some(ev) = translate(event::read()?) {
                    let actions = handle_event(&mut self.state, ev);
                    for action in actions {
                        if self.perform(action, term) {
                            return Ok(());
                        }
                    }
                    self.draw(term)?;
                }
            }
            if last_refresh.elapsed() >= REFRESH {
                self.follow_focus(term);
                self.refresh_windows(term);
                self.draw(term)?;
                last_refresh = Instant::now();
            }
        }
    }

    fn draw(&mut self, term: &mut Term) -> io::Result<()> {
        // ratatui needs the live area to clamp scroll correctly.
        if let Ok(area) = term.size() {
            self.state.width = area.width as usize;
            self.state.height = area.height as usize;
        }
        let width = self.state.width.max(1);
        let height = self.state.height;
        let scroll = self.state.scroll;
        let theme = &self.theme;
        let all = self.state.lines();

        let rows: Vec<Line> = (0..height)
            .map(|row| match all.get(scroll + row) {
                Some(vl) => render_line(&vl.text, vl.style, width, theme),
                None => Line::from(Span::styled(
                    pad("", width),
                    Style::default().bg(theme.bg).fg(theme.fg),
                )),
            })
            .collect();

        term.draw(|frame| {
            let area = frame.area();
            let para = Paragraph::new(rows).style(Style::default().bg(theme.bg).fg(theme.fg));
            frame.render_widget(para, Rect::new(0, 0, area.width, area.height));
        })?;
        Ok(())
    }
}

/// Pad `text` with spaces to exactly `width` display cells (truncating wide
/// content on cell boundaries). Uses unicode-width so emoji/CJK align.
fn pad(text: &str, width: usize) -> String {
    let mut out = String::new();
    let mut cells = 0usize;
    for c in text.chars() {
        let w = unicode_width::UnicodeWidthChar::width(c).unwrap_or(0);
        if cells + w > width {
            break;
        }
        out.push(c);
        cells += w;
    }
    let used = out.width();
    for _ in used..width {
        out.push(' ');
    }
    out
}

fn render_line(text: &str, style: LineStyle, width: usize, theme: &Theme) -> Line<'static> {
    let (bg, fg, bold, dim) = colors_for(style, theme);
    let mut s = Style::default().bg(bg).fg(fg);
    if bold {
        s = s.add_modifier(Modifier::BOLD);
    }
    if dim {
        s = s.add_modifier(Modifier::DIM);
    }
    Line::from(Span::styled(pad(text, width), s))
}

fn colors_for(style: LineStyle, theme: &Theme) -> (Color, Color, bool, bool) {
    match style {
        LineStyle::Blank => (theme.bg, theme.fg, false, false),
        LineStyle::Group => (theme.bg, theme.fg, true, false),
        LineStyle::WindowName { active, bell } => {
            if active {
                (theme.active_bg, theme.active_fg, true, false)
            } else if bell {
                (theme.bell_bg, theme.bell_fg, true, false)
            } else {
                (theme.bg, theme.fg, true, false)
            }
        }
        LineStyle::Detail { active, bell } => {
            let bg = if active {
                theme.active_bg
            } else if bell {
                theme.bell_bg
            } else {
                theme.bg
            };
            (bg, theme.fg, false, true)
        }
        LineStyle::Action => (theme.bg, theme.fg, false, true),
    }
}

/// Translate a crossterm event into our abstract `Event`.
fn translate(ev: CtEvent) -> Option<Event> {
    match ev {
        CtEvent::Key(k) => match k.code {
            KeyCode::Char(c) => Some(Event::Key(c)),
            _ => None,
        },
        CtEvent::Mouse(m) => match m.kind {
            MouseEventKind::Down(MouseButton::Left) => Some(Event::Click { row: m.row }),
            MouseEventKind::Down(MouseButton::Right)
            | MouseEventKind::Down(MouseButton::Middle) => Some(Event::RightClick { row: m.row }),
            MouseEventKind::ScrollUp => Some(Event::ScrollUp),
            MouseEventKind::ScrollDown => Some(Event::ScrollDown),
            _ => None,
        },
        _ => None,
    }
}
