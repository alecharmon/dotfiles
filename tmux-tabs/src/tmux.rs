//! tmux command layer: a `Tmux` trait (so tests can fake it) plus the
//! window/pane gathering logic ported from the Python tool.

use std::collections::{HashMap, HashSet};
use std::io;
use std::path::PathBuf;
use std::process::Command;

use crate::model::{project_group, window_display_name, PrState, PullRequestStatus, Window};

/// Abstraction over running tmux. Real runtime shells out; tests fake it.
pub trait Tmux {
    /// Run `tmux <args>` and return trimmed stdout. Errors propagate.
    fn run(&self, args: &[&str]) -> io::Result<String>;
}

/// Runs the real `tmux` binary.
pub struct RealTmux;

impl Tmux for RealTmux {
    fn run(&self, args: &[&str]) -> io::Result<String> {
        let out = Command::new("tmux").args(args).output()?;
        if !out.status.success() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!(
                    "tmux {:?} failed: {}",
                    args,
                    String::from_utf8_lossy(&out.stderr).trim()
                ),
            ));
        }
        Ok(String::from_utf8_lossy(&out.stdout)
            .trim_end_matches('\n')
            .to_string())
    }
}

/// Environment context resolved once at startup.
#[derive(Debug, Clone)]
pub struct Ctx {
    pub home: String,
    pub cwd: String,
    pub cache_home: PathBuf,
}

impl Ctx {
    pub fn from_env() -> Self {
        let home = std::env::var("HOME").unwrap_or_default();
        let cwd = std::env::current_dir()
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_default();
        let cache_home = std::env::var_os("XDG_CACHE_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(&home).join(".cache"));
        Ctx {
            home,
            cwd,
            cache_home,
        }
    }

    pub fn agent_ready_dir(&self) -> PathBuf {
        self.cache_home.join("dotfiles").join("agent-ready")
    }

    pub fn descriptions_dir(&self, session_id: &str) -> PathBuf {
        self.cache_home
            .join("dotfiles")
            .join("tmux-tabs")
            .join("descriptions")
            .join(session_id.trim_start_matches('$'))
    }

    pub fn pr_status_dir(&self) -> PathBuf {
        self.cache_home.join("dotfiles").join("pr-status")
    }
}

pub fn display<T: Tmux>(t: &T, fmt: &str) -> String {
    t.run(&["display-message", "-p", fmt]).unwrap_or_default()
}

pub fn pane_var<T: Tmux>(t: &T, pane: &str, fmt: &str) -> String {
    t.run(&["display-message", "-p", "-t", pane, fmt])
        .unwrap_or_default()
}

#[derive(Debug, Clone)]
pub struct PaneRecord {
    pub id: String,
    pub left: i64,
    pub top: i64,
    pub active: bool,
    pub command: String,
    pub title: String,
    pub path: String,
}

fn is_sidebar_pane(command: &str, title: &str, path: &str, home: &str) -> bool {
    title == "tmux-tabs"
        || (command == "Python" && path == format!("{home}/dev"))
        || command == "tmux-tabs"
}

pub fn pane_records<T: Tmux>(t: &T, win_index: &str) -> Vec<PaneRecord> {
    let fmt = "#{pane_id}|#{pane_left}|#{pane_top}|#{pane_active}|#{pane_current_command}|#{pane_title}|#{pane_current_path}";
    let out = t
        .run(&["list-panes", "-t", win_index, "-F", fmt])
        .unwrap_or_default();
    let mut panes = Vec::new();
    for line in out.lines() {
        let parts: Vec<&str> = line.splitn(7, '|').collect();
        if parts.len() < 7 {
            continue;
        }
        panes.push(PaneRecord {
            id: parts[0].to_string(),
            left: parts[1].parse().unwrap_or(0),
            top: parts[2].parse().unwrap_or(0),
            active: parts[3] == "1",
            command: parts[4].to_string(),
            title: parts[5].to_string(),
            path: parts[6].to_string(),
        });
    }
    panes
}

fn content_panes<'a>(panes: &'a [PaneRecord], home: &str) -> Vec<&'a PaneRecord> {
    let filtered: Vec<&PaneRecord> = panes
        .iter()
        .filter(|p| !is_sidebar_pane(&p.command, &p.title, &p.path, home))
        .collect();
    if filtered.is_empty() {
        panes.iter().collect()
    } else {
        filtered
    }
}

fn representative_pane<'a>(panes: &'a [PaneRecord], home: &str) -> Option<&'a PaneRecord> {
    let candidates = content_panes(panes, home);
    candidates
        .iter()
        .find(|p| p.active)
        .or_else(|| candidates.first())
        .copied()
}

pub fn leftmost_active_pane<T: Tmux>(t: &T, win_index: &str, home: &str) -> String {
    let panes = pane_records(t, win_index);
    let mut content = content_panes(&panes, home);
    content.sort_by_key(|p| (p.left, p.top));
    content.first().map(|p| p.id.clone()).unwrap_or_default()
}

pub fn active_pane<T: Tmux>(t: &T, win_index: &str, home: &str) -> String {
    let panes = pane_records(t, win_index);
    representative_pane(&panes, home)
        .map(|p| p.id.clone())
        .unwrap_or_default()
}

/// `window_index -> [content pane ids]`, excluding the sidebar pane.
pub fn pane_ids_by_window<T: Tmux>(t: &T, home: &str) -> HashMap<String, Vec<String>> {
    let fmt =
        "#{window_index}|#{pane_id}|#{pane_current_command}|#{pane_title}|#{pane_current_path}";
    let out = t.run(&["list-panes", "-s", "-F", fmt]).unwrap_or_default();
    let mut map: HashMap<String, Vec<String>> = HashMap::new();
    for line in out.lines() {
        let parts: Vec<&str> = line.splitn(5, '|').collect();
        if parts.len() < 5 {
            continue;
        }
        let (win, id, command, title, path) = (parts[0], parts[1], parts[2], parts[3], parts[4]);
        if !is_sidebar_pane(command, title, path, home) {
            map.entry(win.to_string()).or_default().push(id.to_string());
        }
    }
    map
}

fn live_pane_ids(window_panes: &HashMap<String, Vec<String>>) -> HashSet<String> {
    window_panes.values().flatten().cloned().collect()
}

/// Panes flagged "ready" by the agent-ready marker files that are still live.
pub fn ready_panes(ctx: &Ctx, live: &HashSet<String>) -> HashSet<String> {
    let root = ctx.agent_ready_dir();
    let mut ready = HashSet::new();
    let entries = match std::fs::read_dir(&root) {
        Ok(e) => e,
        Err(_) => return ready,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let text = match std::fs::read_to_string(&path) {
            Ok(t) => t,
            Err(_) => continue,
        };
        let pane = json_string_field(&text, "pane");
        match pane {
            Some(p) if live.contains(&p) => {
                ready.insert(p);
            }
            Some(_) => {
                let _ = std::fs::remove_file(&path);
            }
            None => {}
        }
    }
    ready
}

/// Minimal extractor for `"key": "value"` from a flat JSON object — avoids a
/// serde dependency for the couple of fields we read.
fn json_string_field(text: &str, key: &str) -> Option<String> {
    let needle = format!("\"{key}\"");
    let start = text.find(&needle)? + needle.len();
    let rest = &text[start..];
    let colon = rest.find(':')?;
    let after = &rest[colon + 1..];
    let q1 = after.find('"')?;
    let after_q = &after[q1 + 1..];
    let mut value = String::new();
    let mut chars = after_q.chars();
    while let Some(c) = chars.next() {
        match c {
            '"' => return Some(value),
            '\\' => {
                if let Some(n) = chars.next() {
                    value.push(n);
                }
            }
            other => value.push(other),
        }
    }
    None
}

fn json_u64_field(text: &str, key: &str) -> Option<u64> {
    let needle = format!("\"{key}\"");
    let start = text.find(&needle)? + needle.len();
    let rest = &text[start..];
    let colon = rest.find(':')?;
    let digits: String = rest[colon + 1..]
        .chars()
        .skip_while(|c| c.is_whitespace())
        .take_while(|c| c.is_ascii_digit())
        .collect();
    digits.parse().ok()
}

fn json_bool_field(text: &str, key: &str) -> Option<bool> {
    let needle = format!("\"{key}\"");
    let start = text.find(&needle)? + needle.len();
    let rest = &text[start..];
    let colon = rest.find(':')?;
    let value = rest[colon + 1..].trim_start();
    if value.starts_with("true") {
        Some(true)
    } else if value.starts_with("false") {
        Some(false)
    } else {
        None
    }
}

fn ready_window_indexes(
    window_panes: &HashMap<String, Vec<String>>,
    ready: &HashSet<String>,
) -> HashSet<String> {
    window_panes
        .iter()
        .filter(|(_, panes)| panes.iter().any(|p| ready.contains(p)))
        .map(|(w, _)| w.clone())
        .collect()
}

/// Build the full window list for the sidebar (port of `tmux_windows`).
pub fn tmux_windows<T: Tmux>(t: &T, ctx: &Ctx) -> Vec<Window> {
    let window_panes = pane_ids_by_window(t, &ctx.home);
    let live = live_pane_ids(&window_panes);
    let ready = ready_panes(ctx, &live);
    let ready_windows = ready_window_indexes(&window_panes, &ready);

    let session_id = display(t, "#{session_id}");
    let fmt = "#{window_index}|#{window_name}|#{window_bell_flag}|#{window_flags}|#{pane_title}|#{pane_current_path}";
    let out = t.run(&["list-windows", "-F", fmt]).unwrap_or_default();

    let mut windows = Vec::new();
    for line in out.lines() {
        let parts: Vec<&str> = line.splitn(6, '|').collect();
        if parts.len() < 6 {
            continue;
        }
        let (idx, name, bell, flags, title, path) =
            (parts[0], parts[1], parts[2], parts[3], parts[4], parts[5]);

        let panes = pane_records(t, idx);
        let rep = representative_pane(&panes, &ctx.home);
        let display_path = rep
            .map(|p| p.path.clone())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| path.to_string());
        let display_title = rep
            .map(|p| p.title.clone())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| title.to_string());
        let command = rep.map(|p| p.command.clone()).unwrap_or_default();
        let bell_flag = bell == "1";

        windows.push(Window {
            index: idx.to_string(),
            name: window_display_name(name, &command, &display_path),
            bell: bell_flag,
            ready: bell_flag || ready_windows.contains(idx),
            active: flags.contains('*'),
            last: flags.contains('-'),
            title: display_title,
            path: display_path.clone(),
            group: project_group(&display_path, &ctx.home, &ctx.cwd),
            panes: window_panes.get(idx).cloned().unwrap_or_default(),
            description: description_for_window(ctx, &session_id, idx),
            pr: rep.and_then(|p| pr_for_pane(ctx, &p.id)),
        });
    }
    windows
}

fn description_for_window(ctx: &Ctx, session_id: &str, win_index: &str) -> String {
    let path = ctx.descriptions_dir(session_id).join(format!(
        "{}-{}.json",
        session_id.trim_start_matches('$'),
        win_index
    ));
    match std::fs::read_to_string(&path) {
        Ok(text) => json_string_field(&text, "description").unwrap_or_default(),
        Err(_) => String::new(),
    }
}

fn pr_for_pane(ctx: &Ctx, pane: &str) -> Option<PullRequestStatus> {
    let path = ctx
        .pr_status_dir()
        .join(format!("{}.json", pane.trim_start_matches('%')));
    let text = std::fs::read_to_string(&path).ok()?;
    let draft = json_bool_field(&text, "draft").unwrap_or(false);
    let state = json_string_field(&text, "displayState")
        .map(|s| PrState::from_cache(&s))
        .unwrap_or(if draft { PrState::Draft } else { PrState::Open });
    Some(PullRequestStatus {
        number: json_u64_field(&text, "number").unwrap_or_default(),
        title: json_string_field(&text, "title").unwrap_or_default(),
        url: json_string_field(&text, "url").unwrap_or_default(),
        draft,
        state,
    })
}

/// Clear agent-ready markers for the given panes.
pub fn clear_ready_panes(ctx: &Ctx, panes: &[String]) {
    for pane in panes {
        let file = ctx
            .agent_ready_dir()
            .join(format!("{}.json", pane.trim_start_matches('%')));
        let _ = std::fs::remove_file(file);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_field_extraction() {
        let text = r#"{ "pane": "%3", "state": "ready" }"#;
        assert_eq!(json_string_field(text, "pane").as_deref(), Some("%3"));
        assert_eq!(json_string_field(text, "state").as_deref(), Some("ready"));
        assert_eq!(json_string_field(text, "missing"), None);
    }

    #[test]
    fn json_scalar_extraction() {
        let text = r#"{ "number": 42, "draft": false, "title": "Show PRs" }"#;
        assert_eq!(json_u64_field(text, "number"), Some(42));
        assert_eq!(json_bool_field(text, "draft"), Some(false));
        assert_eq!(json_u64_field(text, "missing"), None);
    }

    #[test]
    fn sidebar_pane_detection() {
        assert!(is_sidebar_pane("zsh", "tmux-tabs", "/x", "/home/u"));
        assert!(is_sidebar_pane("Python", "x", "/home/u/dev", "/home/u"));
        assert!(is_sidebar_pane("tmux-tabs", "x", "/x", "/home/u"));
        assert!(!is_sidebar_pane("zsh", "x", "/x", "/home/u"));
    }
}
