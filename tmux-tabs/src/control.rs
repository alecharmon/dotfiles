//! Sidebar control commands: toggle on/off and move between windows.
//! These are the `--select` and no-arg (toggle) entry points, plus the helper
//! the runtime uses to follow focus.

use crate::model::{
    adjacent_sidebar_window_with_sections, sidebar_width_with_config, Direction,
    SidebarWidthConfig, Window,
};
use crate::sections;
use crate::tmux::{active_pane, display, leftmost_active_pane, pane_var, tmux_windows, Ctx, Tmux};

/// Find the sidebar pane in the current session, if present.
pub fn sidebar_pane_id<T: Tmux>(t: &T) -> String {
    let current_session = display(t, "#{session_id}");
    let out = t
        .run(&[
            "list-panes",
            "-a",
            "-F",
            "#{session_id}|#{pane_id}|#{pane_title}",
        ])
        .unwrap_or_default();
    for line in out.lines() {
        let parts: Vec<&str> = line.splitn(3, '|').collect();
        if parts.len() == 3 && parts[0] == current_session && parts[2] == "tmux-tabs" {
            return parts[1].to_string();
        }
    }
    String::new()
}

/// Move the sidebar pane to the left edge of `win_index` and focus that window.
pub fn move_sidebar_to_window<T: Tmux>(
    t: &T,
    ctx: &Ctx,
    sidebar_pane: &str,
    win_index: &str,
    windows: &[Window],
) {
    if sidebar_pane.is_empty() {
        return;
    }
    let target_pane = leftmost_active_pane(t, win_index, &ctx.home);
    let focus_pane = active_pane(t, win_index, &ctx.home);
    let target_width: usize = pane_var(t, &target_pane, "#{window_width}")
        .parse()
        .unwrap_or(0);
    let width_config = sidebar_width_config(t);
    let width = current_sidebar_width(t, sidebar_pane)
        .map(|w| clamp_sidebar_width(w, target_width, width_config))
        .unwrap_or_else(|| sidebar_width_with_config(windows, target_width, width_config));
    let width_s = width.to_string();
    let _ = t.run(&[
        "move-pane",
        "-h",
        "-l",
        &width_s,
        "-s",
        sidebar_pane,
        "-t",
        &target_pane,
        "-d",
        "-b",
    ]);
    let _ = t.run(&["select-window", "-t", win_index]);
    if !focus_pane.is_empty() {
        let _ = t.run(&["select-pane", "-t", &focus_pane]);
    }
}

fn current_sidebar_width<T: Tmux>(t: &T, sidebar_pane: &str) -> Option<usize> {
    pane_var(t, sidebar_pane, "#{pane_width}")
        .trim()
        .parse::<usize>()
        .ok()
        .filter(|w| *w > 0)
}

fn clamp_sidebar_width(width: usize, target_window_width: usize, cfg: SidebarWidthConfig) -> usize {
    let min_width = cfg.min.max(1);
    let max_width = cfg
        .max
        .max(min_width)
        .min((target_window_width / 4).max(min_width));
    width.max(min_width).min(max_width)
}

/// `--select prev|next`: move the sidebar to the adjacent grouped window.
pub fn select_adjacent_sidebar_window<T: Tmux>(t: &T, ctx: &Ctx, dir: Direction) {
    let windows = tmux_windows(t, ctx);
    let current = display(t, "#{window_index}");
    let session_id = display(t, "#{session_id}");
    let layout = sections::load(&ctx.sections_path(&session_id));
    let target = adjacent_sidebar_window_with_sections(&windows, &layout, &current, dir);
    if !target.is_empty() && target != current {
        let sidebar = sidebar_pane_id(t);
        if !sidebar.is_empty() {
            move_sidebar_to_window(t, ctx, &sidebar, &target, &windows);
        } else {
            let _ = t.run(&["select-window", "-t", &target]);
        }
    }
}

/// No-arg entry point: kill the sidebar if open, else spawn it.
pub fn sidebar_width_config<T: Tmux>(t: &T) -> SidebarWidthConfig {
    let default = SidebarWidthConfig::default();
    let read = |name: &str, fallback: usize| -> usize {
        t.run(&["show-option", "-gvq", name])
            .ok()
            .and_then(|v| v.trim().parse::<usize>().ok())
            .filter(|v| *v > 0)
            .unwrap_or(fallback)
    };

    SidebarWidthConfig {
        min: read("@tmux-tabs-min-width", default.min),
        max: read("@tmux-tabs-max-width", default.max),
    }
}

pub fn toggle_sidebar<T: Tmux>(t: &T, ctx: &Ctx, self_exe: &str) {
    let current_session = display(t, "#{session_id}");
    let panes = t
        .run(&[
            "list-panes",
            "-a",
            "-F",
            "#{session_id}|#{pane_id}|#{pane_title}",
        ])
        .unwrap_or_default();
    for line in panes.lines() {
        let parts: Vec<&str> = line.splitn(3, '|').collect();
        if parts.len() == 3 && parts[0] == current_session && parts[2] == "tmux-tabs" {
            let _ = t.run(&["kill-pane", "-t", parts[1]]);
            return;
        }
    }

    let windows = tmux_windows(t, ctx);
    let current_width: usize = display(t, "#{window_width}").parse().unwrap_or(0);
    let width = sidebar_width_with_config(&windows, current_width, sidebar_width_config(t));
    let width_s = width.to_string();
    let current_window = display(t, "#{window_index}");
    let target_pane = leftmost_active_pane(t, &current_window, &ctx.home);
    let cmd = format!("{self_exe} --sidebar");
    let _ = t.run(&[
        "split-window",
        "-h",
        "-l",
        &width_s,
        "-t",
        &target_pane,
        "-b",
        &cmd,
    ]);
}

#[cfg(test)]
mod tests {
    use std::io;

    use super::*;

    use std::cell::RefCell;

    struct FakeTmux {
        calls: RefCell<Vec<String>>,
        sidebar_width: usize,
    }

    impl FakeTmux {
        fn new(sidebar_width: usize) -> Self {
            Self {
                calls: RefCell::new(Vec::new()),
                sidebar_width,
            }
        }
    }

    impl Tmux for FakeTmux {
        fn run(&self, args: &[&str]) -> io::Result<String> {
            self.calls.borrow_mut().push(args.join(" "));
            match args {
                ["show-option", "-gvq", "@tmux-tabs-min-width"] => Ok("24".to_string()),
                ["show-option", "-gvq", "@tmux-tabs-max-width"] => Ok("nope".to_string()),
                ["list-panes", "-t", "2", "-F", _] => {
                    Ok("%2|0|0|1|zsh|content|/home/alec/dev/proj".to_string())
                }
                ["display-message", "-p", "-t", "%sidebar", "#{pane_width}"] => {
                    Ok(self.sidebar_width.to_string())
                }
                ["display-message", "-p", "-t", "%2", "#{window_width}"] => Ok("120".to_string()),
                _ => Ok(String::new()),
            }
        }
    }

    #[test]
    fn sidebar_width_config_reads_tmux_options_with_defaults() {
        assert_eq!(
            sidebar_width_config(&FakeTmux::new(24)),
            SidebarWidthConfig { min: 24, max: 48 }
        );
    }

    #[test]
    fn moving_existing_sidebar_preserves_current_width() {
        let t = FakeTmux::new(24);
        let ctx = Ctx {
            home: "/home/alec".to_string(),
            cwd: "/home/alec".to_string(),
            cache_home: "/tmp".into(),
        };
        let windows = vec![Window {
            index: "2".to_string(),
            name: "a-really-quite-long-window-name-that-would-autogrow".to_string(),
            group: "proj".to_string(),
            ..Default::default()
        }];

        move_sidebar_to_window(&t, &ctx, "%sidebar", "2", &windows);

        assert!(
            t.calls
                .borrow()
                .iter()
                .any(|c| c == "move-pane -h -l 24 -s %sidebar -t %2 -d -b"),
            "calls: {:?}",
            t.calls.borrow()
        );
    }

    #[test]
    fn moving_existing_sidebar_width_clamps_below_minimum() {
        let t = FakeTmux::new(10);
        let ctx = Ctx {
            home: "/home/alec".to_string(),
            cwd: "/home/alec".to_string(),
            cache_home: "/tmp".into(),
        };
        let windows = vec![Window {
            index: "2".to_string(),
            name: "short".to_string(),
            group: "proj".to_string(),
            ..Default::default()
        }];

        move_sidebar_to_window(&t, &ctx, "%sidebar", "2", &windows);

        assert!(
            t.calls
                .borrow()
                .iter()
                .any(|c| c == "move-pane -h -l 24 -s %sidebar -t %2 -d -b"),
            "calls: {:?}",
            t.calls.borrow()
        );
    }

    #[test]
    fn moving_existing_sidebar_width_clamps_above_max_for_target_window() {
        let t = FakeTmux::new(200);
        let ctx = Ctx {
            home: "/home/alec".to_string(),
            cwd: "/home/alec".to_string(),
            cache_home: "/tmp".into(),
        };
        let windows = vec![Window {
            index: "2".to_string(),
            name: "a-really-quite-long-window-name-that-would-autogrow".to_string(),
            group: "proj".to_string(),
            ..Default::default()
        }];

        move_sidebar_to_window(&t, &ctx, "%sidebar", "2", &windows);

        assert!(
            t.calls
                .borrow()
                .iter()
                .any(|c| c == "move-pane -h -l 30 -s %sidebar -t %2 -d -b"),
            "calls: {:?}",
            t.calls.borrow()
        );
    }
}
