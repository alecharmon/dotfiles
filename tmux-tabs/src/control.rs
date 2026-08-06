//! Sidebar control commands: toggle on/off and move between windows.
//! These are the `--select` and no-arg (toggle) entry points, plus the helper
//! the runtime uses to follow focus.

use crate::model::{adjacent_sidebar_window_with_sections, Direction, DEFAULT_WIDTH};
use crate::sections;
use crate::tmux::{active_pane, display, leftmost_active_pane, tmux_windows, Ctx, Tmux};

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
pub fn move_sidebar_to_window<T: Tmux>(t: &T, ctx: &Ctx, sidebar_pane: &str, win_index: &str) {
    if sidebar_pane.is_empty() {
        return;
    }
    let target_pane = leftmost_active_pane(t, win_index, &ctx.home);
    let focus_pane = active_pane(t, win_index, &ctx.home);
    let width_s = sidebar_width(t).to_string();
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
    // move-pane does not always honour -l exactly; force the width back.
    let _ = t.run(&["resize-pane", "-t", sidebar_pane, "-x", &width_s]);
    let _ = t.run(&["select-window", "-t", win_index]);
    if !focus_pane.is_empty() {
        let _ = t.run(&["select-pane", "-t", &focus_pane]);
    }
}

/// The sidebar's fixed width: `@tmux-tabs-width`, else `DEFAULT_WIDTH`.
pub fn sidebar_width<T: Tmux>(t: &T) -> usize {
    t.run(&["show-option", "-gvq", "@tmux-tabs-width"])
        .ok()
        .and_then(|v| v.trim().parse::<usize>().ok())
        .filter(|w| *w > 0)
        .unwrap_or(DEFAULT_WIDTH)
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
            move_sidebar_to_window(t, ctx, &sidebar, &target);
        } else {
            let _ = t.run(&["select-window", "-t", &target]);
        }
    }
}

/// No-arg entry point: kill the sidebar if open, else spawn it.
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

    let width_s = sidebar_width(t).to_string();
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
    use std::cell::RefCell;
    use std::io;

    use super::*;

    struct FakeTmux {
        calls: RefCell<Vec<String>>,
        configured_width: Option<usize>,
    }

    impl FakeTmux {
        fn new(configured_width: Option<usize>) -> Self {
            Self {
                calls: RefCell::new(Vec::new()),
                configured_width,
            }
        }
    }

    impl Tmux for FakeTmux {
        fn run(&self, args: &[&str]) -> io::Result<String> {
            self.calls.borrow_mut().push(args.join(" "));
            match args {
                ["show-option", "-gvq", "@tmux-tabs-width"] => Ok(self
                    .configured_width
                    .map(|w| w.to_string())
                    .unwrap_or_default()),
                ["list-panes", "-t", "2", "-F", _] => {
                    Ok("%2|0|0|1|zsh|content|/home/alec/dev/proj|/dev/ttys001".to_string())
                }
                _ => Ok(String::new()),
            }
        }
    }

    fn ctx() -> Ctx {
        Ctx {
            home: "/home/alec".to_string(),
            cwd: "/home/alec".to_string(),
            cache_home: "/tmp".into(),
        }
    }

    #[test]
    fn width_comes_from_the_tmux_option_or_the_default() {
        assert_eq!(sidebar_width(&FakeTmux::new(Some(40))), 40);
        assert_eq!(sidebar_width(&FakeTmux::new(None)), DEFAULT_WIDTH);
    }

    #[test]
    fn moving_the_sidebar_always_uses_the_fixed_width() {
        let t = FakeTmux::new(Some(32));

        move_sidebar_to_window(&t, &ctx(), "%sidebar", "2");

        let calls = t.calls.borrow();
        assert!(
            calls
                .iter()
                .any(|c| c == "move-pane -h -l 32 -s %sidebar -t %2 -d -b"),
            "calls: {:?}",
            calls
        );
        assert!(
            calls.iter().any(|c| c == "resize-pane -t %sidebar -x 32"),
            "calls: {:?}",
            calls
        );
    }
}
