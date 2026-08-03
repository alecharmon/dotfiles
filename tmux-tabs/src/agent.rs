use crate::model::Window;
use crate::sections::SectionLayout;
use crate::tmux::{active_pane, tmux_windows, Ctx, Tmux};

fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\r' => {}
            other => out.push(other),
        }
    }
    out
}

pub fn windows_json(windows: &[Window]) -> String {
    let mut out = String::from("[\n");
    for (i, w) in windows.iter().enumerate() {
        if i > 0 {
            out.push_str(",\n");
        }
        out.push_str(&format!(
            "  {{\"index\":\"{}\",\"window_id\":\"{}\",\"tab_id\":\"{}\",\"name\":\"{}\",\"description\":\"{}\",\"title\":\"{}\",\"path\":\"{}\",\"pane\":\"{}\",\"command\":\"{}\",\"active\":{}}}",
            json_escape(&w.index),
            json_escape(&w.window_id),
            json_escape(&w.tab_id),
            json_escape(&w.name),
            json_escape(&w.description),
            json_escape(&w.title),
            json_escape(&w.path),
            json_escape(&w.pane),
            json_escape(&w.command),
            w.active
        ));
    }
    out.push_str("\n]\n");
    out
}

fn normalize_ref(reference: &str) -> String {
    reference
        .strip_prefix("@tab:")
        .unwrap_or(reference)
        .trim_matches('"')
        .trim()
        .to_lowercase()
}

pub fn find_window<'a>(windows: &'a [Window], reference: &str) -> Option<&'a Window> {
    let query = normalize_ref(reference);
    if query.is_empty() {
        return None;
    }
    windows
        .iter()
        .find(|w| w.index == query)
        .or_else(|| {
            windows.iter().find(|w| {
                [
                    &w.name,
                    &w.description,
                    &w.title,
                    &w.path,
                    &w.tab_id,
                    &w.window_id,
                ]
                .iter()
                .any(|value| value.to_lowercase() == query)
            })
        })
        .or_else(|| {
            windows.iter().find(|w| {
                [
                    &w.name,
                    &w.description,
                    &w.title,
                    &w.path,
                    &w.tab_id,
                    &w.window_id,
                ]
                .iter()
                .any(|value| value.to_lowercase().contains(&query))
            })
        })
}

pub fn find_json(windows: &[Window], reference: &str) -> String {
    match find_window(windows, reference) {
        Some(w) => windows_json(std::slice::from_ref(w)),
        None => "null\n".to_string(),
    }
}

pub fn capture_tab<T: Tmux>(t: &T, ctx: &Ctx, reference: &str) -> Option<String> {
    let windows = tmux_windows(t, ctx);
    let window = find_window(&windows, reference)?;
    let pane = if window.pane.is_empty() {
        active_pane(t, &window.index, &ctx.home)
    } else {
        window.pane.clone()
    };
    if pane.is_empty() {
        return None;
    }
    t.run(&["capture-pane", "-p", "-t", &pane, "-S", "-200"])
        .ok()
}

pub fn add_section_window_by_ref(
    layout: &mut SectionLayout,
    windows: &[Window],
    section: &str,
    reference: &str,
) -> bool {
    let Some(window) = find_window(windows, reference) else {
        return false;
    };
    if window.window_id.is_empty() {
        return false;
    }
    layout.add_window(section, &window.window_id);
    true
}

pub fn remove_section_window_by_ref(
    layout: &mut SectionLayout,
    windows: &[Window],
    section: &str,
    reference: &str,
) -> bool {
    let Some(window) = find_window(windows, reference) else {
        return false;
    };
    if window.window_id.is_empty() {
        return false;
    }
    layout.remove_window(section, &window.window_id);
    true
}

pub fn send_to_tab<T: Tmux>(t: &T, ctx: &Ctx, reference: &str, text: &str, enter: bool) -> bool {
    let windows = tmux_windows(t, ctx);
    let Some(window) = find_window(&windows, reference) else {
        return false;
    };
    let pane = if window.pane.is_empty() {
        active_pane(t, &window.index, &ctx.home)
    } else {
        window.pane.clone()
    };
    if pane.is_empty() {
        return false;
    }
    if t.run(&["set-buffer", "--", text]).is_err() {
        return false;
    }
    if t.run(&["paste-buffer", "-t", &pane]).is_err() {
        return false;
    }
    if enter {
        let _ = t.run(&["send-keys", "-t", &pane, "Enter"]);
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{PrState, PullRequestStatus};
    use std::cell::RefCell;
    use std::io;

    struct FakeTmux {
        calls: RefCell<Vec<String>>,
        fail_set_buffer: bool,
    }

    impl FakeTmux {
        fn new(fail_set_buffer: bool) -> Self {
            Self {
                calls: RefCell::new(Vec::new()),
                fail_set_buffer,
            }
        }
    }

    impl Tmux for FakeTmux {
        fn run(&self, args: &[&str]) -> io::Result<String> {
            self.calls.borrow_mut().push(args.join(" "));
            match args {
                ["list-panes", "-s", "-F", _] => Ok("1|%1|zsh|agent|/repo".to_string()),
                ["display-message", "-p", "#{session_id}"] => Ok("$1".to_string()),
                ["list-windows", "-F", _] => Ok("1|@1|tab_1|Agent|0|*|agent|/repo".to_string()),
                ["list-panes", "-t", "1", "-F", _] => Ok("%1|0|0|1|zsh|agent|/repo".to_string()),
                ["display-message", "-p", "-t", "%1", "#{window_width}"] => Ok("120".to_string()),
                ["set-buffer", ..] if self.fail_set_buffer => {
                    Err(io::Error::new(io::ErrorKind::Other, "set failed"))
                }
                _ => Ok(String::new()),
            }
        }
    }

    fn test_ctx() -> Ctx {
        Ctx {
            home: "/home/alec".to_string(),
            cwd: "/home/alec".to_string(),
            cache_home: std::env::temp_dir().join("tmux-tabs-agent-tests"),
        }
    }

    fn window(index: &str, name: &str, description: &str, path: &str) -> Window {
        Window {
            index: index.to_string(),
            window_id: format!("@{index}"),
            tab_id: format!("tab_{index}"),
            name: name.to_string(),
            bell: false,
            ready: false,
            active: false,
            last: false,
            title: "zsh".to_string(),
            path: path.to_string(),
            group: "repo".to_string(),
            pane: format!("%{index}"),
            panes: vec![format!("%{index}")],
            command: "pi".to_string(),
            description: description.to_string(),
            pr: Some(PullRequestStatus {
                number: 1,
                title: "PR".to_string(),
                url: "https://example.com".to_string(),
                draft: false,
                state: PrState::Open,
            }),
        }
    }

    #[test]
    fn list_json_exposes_agent_addressable_fields() {
        let json = windows_json(&[window("2", "Sidebar Naming", "renaming tmux tabs", "/repo")]);

        assert!(json.contains("\"index\":\"2\""));
        assert!(json.contains("\"name\":\"Sidebar Naming\""));
        assert!(json.contains("\"description\":\"renaming tmux tabs\""));
        assert!(json.contains("\"pane\":\"%2\""));
        assert!(json.contains("\"tab_id\":\"tab_2\""));
    }

    #[test]
    fn finds_window_by_tab_mention_name_description_or_index() {
        let windows = vec![
            window("1", "Tests", "running tests", "/repo"),
            window("2", "Sidebar Naming", "renaming tmux tabs", "/repo/sidebar"),
        ];

        assert_eq!(find_window(&windows, "@tab:sidebar").unwrap().index, "2");
        assert_eq!(find_window(&windows, "renaming").unwrap().index, "2");
        assert_eq!(find_window(&windows, "1").unwrap().name, "Tests");
        assert!(find_window(&windows, "missing").is_none());
    }

    #[test]
    fn section_add_and_remove_resolve_references_to_window_ids() {
        let windows = vec![
            window("1", "Tests", "running tests", "/repo"),
            window("2", "Sidebar Naming", "renaming tmux tabs", "/repo/sidebar"),
        ];
        let mut layout = SectionLayout::default();

        assert!(add_section_window_by_ref(
            &mut layout,
            &windows,
            "Review",
            "@tab:sidebar"
        ));
        assert_eq!(layout.sections[0].windows, vec!["@2".to_string()]);

        assert!(remove_section_window_by_ref(
            &mut layout,
            &windows,
            "Review",
            "renaming"
        ));
        assert!(layout.sections[0].windows.is_empty());
    }

    #[test]
    fn send_to_tab_fails_if_setting_buffer_fails() {
        let t = FakeTmux::new(true);
        let sent = send_to_tab(&t, &test_ctx(), "Agent", "hello", true);

        assert!(!sent);
        assert!(
            !t.calls
                .borrow()
                .iter()
                .any(|c| c.starts_with("paste-buffer")),
            "calls: {:?}",
            t.calls.borrow()
        );
    }
}
