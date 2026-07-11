//! Pure data model and layout logic for the tmux sidebar.
//!
//! Everything in this module is free of I/O so it can be exercised directly
//! from integration tests. The runtime layer (`tmux`, `runtime`) feeds these
//! functions real data and renders/acts on their output.

use std::path::{Component, Path, PathBuf};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum PrState {
    Draft,
    CiRunning,
    CiFailed,
    Changes,
    Ready,
    Merged,
    Closed,
    #[default]
    Open,
}

impl PrState {
    pub fn label(self) -> &'static str {
        match self {
            PrState::Draft => "draft",
            PrState::CiRunning => "CI…",
            PrState::CiFailed => "failed",
            PrState::Changes => "changes",
            PrState::Ready => "ready",
            PrState::Merged => "merged",
            PrState::Closed => "closed",
            PrState::Open => "open",
        }
    }

    pub fn from_cache(value: &str) -> Self {
        match value {
            "draft" => PrState::Draft,
            "ci_running" => PrState::CiRunning,
            "ci_failed" => PrState::CiFailed,
            "changes" => PrState::Changes,
            "ready" => PrState::Ready,
            "merged" => PrState::Merged,
            "closed" => PrState::Closed,
            _ => PrState::Open,
        }
    }
}

/// A single tmux window as shown in the sidebar.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PullRequestStatus {
    pub number: u64,
    pub title: String,
    pub url: String,
    pub draft: bool,
    pub state: PrState,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Window {
    pub index: String,
    pub name: String,
    pub bell: bool,
    pub ready: bool,
    pub active: bool,
    pub last: bool,
    pub title: String,
    pub path: String,
    pub group: String,
    pub pane: String,
    pub panes: Vec<String>,
    pub description: String,
    pub pr: Option<PullRequestStatus>,
}

pub const ICON_READY: &str = "🔔 ";
pub const ICON_EMPTY: &str = "   ";
pub const DETAIL_INDENT: &str = "   ";
pub const DESCRIPTION_CACHE_MAX: usize = 200;
pub const DESCRIPTION_MIN: usize = 8;
pub const MIN_WIDTH: usize = 18;
pub const MAX_WIDTH: usize = 48;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SidebarWidthConfig {
    pub min: usize,
    pub max: usize,
}

impl Default for SidebarWidthConfig {
    fn default() -> Self {
        Self {
            min: MIN_WIDTH,
            max: MAX_WIDTH,
        }
    }
}

/// Lexically normalise a path: expand a leading `~`, make it absolute relative
/// to `cwd`, and collapse `.`/`..` components without touching the filesystem
/// (the path may not exist).
pub fn normalize_path(input: &str, home: &str, cwd: &str) -> PathBuf {
    let expanded: PathBuf = if input == "~" {
        PathBuf::from(home)
    } else if let Some(rest) = input.strip_prefix("~/") {
        Path::new(home).join(rest)
    } else if input.is_empty() {
        PathBuf::from(cwd)
    } else {
        PathBuf::from(input)
    };

    let absolute = if expanded.is_absolute() {
        expanded
    } else {
        Path::new(cwd).join(expanded)
    };

    let mut out = PathBuf::new();
    for comp in absolute.components() {
        match comp {
            Component::ParentDir => {
                out.pop();
            }
            Component::CurDir => {}
            other => out.push(other.as_os_str()),
        }
    }
    out
}

/// Map a working directory to its project group, mirroring the Python
/// `project_group`: paths under `~/dev` and `~/worktrees` group by their first
/// component, with `~/dev/flora/<x>` grouping by `<x>`. The roots themselves
/// are `dev` and `worktrees`; anything else is `Other`.
pub fn project_group(path: &str, home: &str, cwd: &str) -> String {
    let roots = [
        (normalize_path("~/dev", home, cwd), "dev"),
        (normalize_path("~/worktrees", home, cwd), "worktrees"),
    ];
    let abs = normalize_path(path, home, cwd);

    let (rel, root_name) = match roots.iter().find_map(|(root, name)| {
        abs.strip_prefix(root).ok().map(|rel| (rel, *name))
    }) {
        Some(found) => found,
        None => return "Other".to_string(),
    };

    let parts: Vec<String> = rel
        .components()
        .filter_map(|c| match c {
            Component::Normal(s) => Some(s.to_string_lossy().into_owned()),
            _ => None,
        })
        .collect();

    if parts.is_empty() {
        return root_name.to_string();
    }
    if parts[0] == "flora" && parts.len() > 1 && !parts[1].is_empty() {
        return parts[1].clone();
    }
    if parts[0].is_empty() {
        return "Other".to_string();
    }
    parts[0].clone()
}

/// Group windows by their `group`, ordering group names case-insensitively
/// with `Other` always last (mirrors `grouped_windows`).
pub fn grouped_windows(windows: &[Window]) -> Vec<(String, Vec<&Window>)> {
    let mut order: Vec<String> = Vec::new();
    let mut groups: std::collections::HashMap<String, Vec<&Window>> =
        std::collections::HashMap::new();
    for w in windows {
        if !groups.contains_key(&w.group) {
            order.push(w.group.clone());
        }
        groups.entry(w.group.clone()).or_default().push(w);
    }

    let mut names: Vec<String> = order.iter().filter(|n| *n != "Other").cloned().collect();
    names.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));
    if groups.contains_key("Other") {
        names.push("Other".to_string());
    }

    names
        .into_iter()
        .map(|name| {
            let ws = groups.remove(&name).unwrap_or_default();
            (name, ws)
        })
        .collect()
}

/// The window indexes in sidebar (grouped) order.
pub fn sidebar_window_order(windows: &[Window]) -> Vec<String> {
    grouped_windows(windows)
        .into_iter()
        .flat_map(|(_, ws)| ws.into_iter().map(|w| w.index.clone()))
        .collect()
}

pub fn pr_refresh_targets(windows: &[Window]) -> Vec<(String, String)> {
    windows
        .iter()
        .filter(|w| !w.pane.is_empty() && !w.path.is_empty())
        .map(|w| (w.pane.clone(), w.path.clone()))
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Prev,
    Next,
}

/// The window index adjacent to `current` in sidebar order, wrapping around.
pub fn adjacent_sidebar_window(windows: &[Window], current: &str, dir: Direction) -> String {
    let order = sidebar_window_order(windows);
    if order.is_empty() {
        return String::new();
    }
    let idx = match order.iter().position(|i| i == current) {
        Some(i) => i,
        None => return order[0].clone(),
    };
    let len = order.len();
    let next = match dir {
        Direction::Prev => (idx + len - 1) % len,
        Direction::Next => (idx + 1) % len,
    };
    order[next].clone()
}

fn char_len(s: &str) -> usize {
    s.chars().count()
}

fn strip_star(title: &str) -> String {
    title.trim_start_matches('✳').trim().to_string()
}

/// Compute the desired sidebar width in cells for the given windows and host
/// window width (mirrors `sidebar_width`).
pub fn sidebar_width(windows: &[Window], window_width: usize) -> usize {
    sidebar_width_with_config(windows, window_width, SidebarWidthConfig::default())
}

pub fn sidebar_width_with_config(
    windows: &[Window],
    window_width: usize,
    config: SidebarWidthConfig,
) -> usize {
    let min_width = config.min.max(1);
    let configured_max = config.max.max(min_width);

    let mut content_width = min_width;
    for (group, group_windows) in grouped_windows(windows) {
        content_width = content_width.max(char_len(&group) + 2);
        for w in group_windows {
            content_width = content_width.max(char_len(&w.name) + 4);
            let title = strip_star(&w.title);
            if !title.is_empty() {
                content_width = content_width.max(char_len(&title) + 6);
            }
        }
    }
    let max_width = configured_max.min((window_width / 4).max(min_width));
    min_width.max(content_width.min(max_width))
}

/// Derive the display name for a window, expanding bare `Python` panes to a
/// `path:command` form (mirrors `window_display_name`).
pub fn window_display_name(tmux_name: &str, command: &str, path: &str) -> String {
    if command == "Python" || tmux_name.ends_with(":Python") {
        let base = Path::new(path)
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| tmux_name.split(':').next().unwrap_or(tmux_name).to_string());
        let cmd = command.trim();
        let cmd = if cmd.is_empty() { "pane" } else { cmd };
        return format!("{base}:{cmd}");
    }
    tmux_name.to_string()
}

fn description_max_for_width(width: usize) -> usize {
    DESCRIPTION_MIN.max(width.saturating_sub(char_len(DETAIL_INDENT) + 2))
}

/// Clip text to fit either the description cache cap (`width == None`) or the
/// sidebar width, appending an ellipsis when truncated.
pub fn clipped_text(text: &str, width: Option<usize>) -> String {
    let text = text.trim();
    let max_len = match width {
        None => DESCRIPTION_CACHE_MAX,
        Some(w) => description_max_for_width(w),
    };
    if char_len(text) <= max_len {
        return text.to_string();
    }
    if max_len == 0 {
        return "…".to_string();
    }
    let kept: String = text.chars().take(max_len - 1).collect();
    format!("{}…", kept.trim_end())
}

pub fn window_icon(w: &Window) -> &'static str {
    if w.ready {
        ICON_READY
    } else {
        ICON_EMPTY
    }
}

/// Action menu labels for a window (mirrors `action_menu_labels`).
pub fn action_menu_labels(has_pr_url: bool) -> Vec<&'static str> {
    if has_pr_url {
        vec!["open PR", "refresh PR", "rename", "kill", "kill + rm worktree", "clear ready"]
    } else {
        vec!["refresh PR", "rename", "kill", "kill + rm worktree", "clear ready"]
    }
}
