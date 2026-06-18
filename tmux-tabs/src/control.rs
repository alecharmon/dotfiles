//! Sidebar control commands: toggle on/off and move between windows.
//! These are the `--select` and no-arg (toggle) entry points, plus the helper
//! the runtime uses to follow focus.

use crate::model::{adjacent_sidebar_window, sidebar_width, Direction, Window};
use crate::tmux::{
    active_pane, display, leftmost_active_pane, pane_var, tmux_windows, Ctx, Tmux,
};

/// Find the sidebar pane in the current session, if present.
pub fn sidebar_pane_id<T: Tmux>(t: &T) -> String {
    let current_session = display(t, "#{session_id}");
    let out = t
        .run(&["list-panes", "-a", "-F", "#{session_id}|#{pane_id}|#{pane_title}"])
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
    let target_width: usize = pane_var(t, &target_pane, "#{window_width}").parse().unwrap_or(0);
    let width = sidebar_width(windows, target_width);
    let width_s = width.to_string();
    let _ = t.run(&[
        "move-pane", "-h", "-l", &width_s, "-s", sidebar_pane, "-t", &target_pane, "-d", "-b",
    ]);
    let _ = t.run(&["select-window", "-t", win_index]);
    if !focus_pane.is_empty() {
        let _ = t.run(&["select-pane", "-t", &focus_pane]);
    }
}

/// `--select prev|next`: move the sidebar to the adjacent grouped window.
pub fn select_adjacent_sidebar_window<T: Tmux>(t: &T, ctx: &Ctx, dir: Direction) {
    let windows = tmux_windows(t, ctx);
    let current = display(t, "#{window_index}");
    let target = adjacent_sidebar_window(&windows, &current, dir);
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
pub fn toggle_sidebar<T: Tmux>(t: &T, ctx: &Ctx, self_exe: &str) {
    let current_session = display(t, "#{session_id}");
    let panes = t
        .run(&["list-panes", "-a", "-F", "#{session_id}|#{pane_id}|#{pane_title}"])
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
    let width = sidebar_width(&windows, current_width);
    let width_s = width.to_string();
    let current_window = display(t, "#{window_index}");
    let target_pane = leftmost_active_pane(t, &current_window, &ctx.home);
    let cmd = format!("{self_exe} --sidebar");
    let _ = t.run(&["split-window", "-h", "-l", &width_s, "-t", &target_pane, "-b", &cmd]);
}
