//! Integration tests for the pure sidebar logic: grouping, layout, and the
//! mouse/scroll/click/key event reducer. No terminal or tmux involved.

use tmux_tabs::app::{handle_event, Action, Event, State};
use tmux_tabs::layout::{build_lines, LineStyle, Target};
use tmux_tabs::model::{
    adjacent_sidebar_window, clipped_text, grouped_windows, project_group, sidebar_width,
    sidebar_window_order, window_display_name, Direction, Window,
};

const HOME: &str = "/home/alec";
const CWD: &str = "/home/alec";

fn win(index: &str, group: &str, name: &str) -> Window {
    Window {
        index: index.to_string(),
        name: name.to_string(),
        group: group.to_string(),
        ..Default::default()
    }
}

// ----- model -----

#[test]
fn project_group_classifies_paths() {
    assert_eq!(project_group("/home/alec/dev", HOME, CWD), "dev");
    assert_eq!(project_group("/home/alec/dev/myrepo", HOME, CWD), "myrepo");
    assert_eq!(project_group("/home/alec/dev/myrepo/sub", HOME, CWD), "myrepo");
    assert_eq!(project_group("/home/alec/dev/flora/api", HOME, CWD), "api");
    assert_eq!(project_group("/home/alec/dev/flora/api/x", HOME, CWD), "api");
    assert_eq!(project_group("/home/alec/dev/flora", HOME, CWD), "flora");
    assert_eq!(project_group("/etc", HOME, CWD), "Other");
    assert_eq!(project_group("~/dev/zed", HOME, CWD), "zed");
}

#[test]
fn groups_order_alphabetically_with_other_last() {
    let windows = vec![
        win("1", "Other", "a"),
        win("2", "zebra", "b"),
        win("3", "Apple", "c"),
        win("4", "Other", "d"),
    ];
    let names: Vec<String> = grouped_windows(&windows).into_iter().map(|(n, _)| n).collect();
    assert_eq!(names, vec!["Apple", "zebra", "Other"]);
}

#[test]
fn sidebar_order_and_adjacency_wraps() {
    let windows = vec![
        win("1", "Apple", "a"),
        win("2", "zebra", "b"),
        win("3", "Other", "c"),
    ];
    assert_eq!(sidebar_window_order(&windows), vec!["1", "2", "3"]);
    assert_eq!(adjacent_sidebar_window(&windows, "1", Direction::Next), "2");
    assert_eq!(adjacent_sidebar_window(&windows, "3", Direction::Next), "1"); // wrap
    assert_eq!(adjacent_sidebar_window(&windows, "1", Direction::Prev), "3"); // wrap
    assert_eq!(adjacent_sidebar_window(&windows, "99", Direction::Next), "1"); // unknown -> first
    assert_eq!(adjacent_sidebar_window(&[], "1", Direction::Next), "");
}

#[test]
fn width_grows_with_content_and_is_capped() {
    let narrow = vec![win("1", "g", "short")];
    assert_eq!(sidebar_width(&narrow, 400), 18); // floor

    let mut w = win("1", "g", "a-really-quite-long-window-name-here");
    w.title = "and an even longer descriptive title goes here too".to_string();
    let wide = sidebar_width(&[w], 400);
    assert!(wide > 18 && wide <= 48, "got {wide}");

    // host width caps the result: 40/4 = 10 -> floored to 18.
    assert_eq!(sidebar_width(&narrow, 40), 18);
}

#[test]
fn display_name_expands_python_panes() {
    assert_eq!(window_display_name("zsh", "zsh", "/x"), "zsh");
    assert_eq!(
        window_display_name("foo:Python", "Python", "/home/alec/dev/proj"),
        "proj:Python"
    );
}

#[test]
fn clipped_text_adds_ellipsis() {
    assert_eq!(clipped_text("hello", Some(20)), "hello");
    let clipped = clipped_text("abcdefghijklmnop", Some(10));
    assert!(clipped.ends_with('…'));
    assert!(clipped.chars().count() <= 10 - 3 /* indent */ + 1);
}

// ----- layout + clicks -----

fn three_window_state() -> State {
    let mut windows = vec![
        win("1", "Apple", "first"),
        win("2", "Apple", "second"),
        win("3", "zebra", "third"),
    ];
    windows[0].active = true;
    State {
        windows,
        width: 30,
        height: 100,
        ..Default::default()
    }
}

#[test]
fn layout_targets_map_back_to_windows() {
    let s = three_window_state();
    let lines = build_lines(&s.windows, "", false, s.width);
    // Every window-name line should target SwitchTo with its index.
    let switch_targets: Vec<String> = lines
        .iter()
        .filter_map(|l| match (&l.style, &l.target) {
            (LineStyle::WindowName { .. }, Some(Target::SwitchTo(i))) => Some(i.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(switch_targets, vec!["1", "2", "3"]);
}

#[test]
fn left_click_switches_to_clicked_window() {
    let mut s = three_window_state();
    // Find the screen row of window "2".
    let lines = s.lines();
    let row = lines
        .iter()
        .position(|l| l.target == Some(Target::SwitchTo("2".into())))
        .unwrap() as u16;
    let actions = handle_event(&mut s, Event::Click { row });
    assert_eq!(actions, vec![Action::SwitchTo("2".into())]);
    assert_eq!(s.current_window, "2");
}

#[test]
fn click_on_blank_does_nothing() {
    let mut s = three_window_state();
    // Row 0 is top padding (blank).
    let actions = handle_event(&mut s, Event::Click { row: 0 });
    assert!(actions.is_empty());
}

#[test]
fn right_click_opens_and_toggles_action_menu() {
    let mut s = three_window_state();
    let row = s
        .lines()
        .iter()
        .position(|l| l.target == Some(Target::SwitchTo("3".into())))
        .unwrap() as u16;

    handle_event(&mut s, Event::RightClick { row });
    assert_eq!(s.menu_window, "3");
    // Menu items now exist in the layout.
    assert!(s
        .lines()
        .iter()
        .any(|l| matches!(&l.target, Some(Target::RunAction { action, .. }) if action == "rename")));

    // Right-clicking the same window again closes it.
    handle_event(&mut s, Event::RightClick { row });
    assert_eq!(s.menu_window, "");
}

#[test]
fn action_menu_kill_requires_confirmation() {
    let mut s = three_window_state();
    s.menu_window = "2".into();

    let kill_row = |s: &State| {
        s.lines()
            .iter()
            .position(|l| {
                matches!(&l.target, Some(Target::RunAction { action, .. }) if action == "kill")
            })
            .unwrap() as u16
    };
    // First "kill" click only arms confirmation, no external action.
    let kr = kill_row(&s);
    let actions = handle_event(&mut s, Event::Click { row: kr });
    assert!(actions.is_empty());
    assert!(s.confirm_kill);

    // Now a "confirm kill" item exists.
    let confirm_row = s
        .lines()
        .iter()
        .position(|l| {
            matches!(&l.target, Some(Target::RunAction { action, .. }) if action == "confirm kill")
        })
        .unwrap() as u16;
    let actions = handle_event(&mut s, Event::Click { row: confirm_row });
    assert_eq!(actions, vec![Action::KillWindow("2".into())]);
    assert_eq!(s.menu_window, "");
    assert!(!s.confirm_kill);
}

#[test]
fn action_menu_rename_and_clear_ready_emit_actions() {
    let mut s = three_window_state();
    s.menu_window = "1".into();
    let row_of = |s: &State, action: &str| {
        s.lines()
            .iter()
            .position(|l| matches!(&l.target, Some(Target::RunAction { action: a, .. }) if a == action))
            .map(|p| p as u16)
    };

    let r = row_of(&s, "rename").unwrap();
    assert_eq!(handle_event(&mut s, Event::Click { row: r }), vec![Action::Rename("1".into())]);
    assert_eq!(s.menu_window, "");

    s.menu_window = "1".into();
    let r = row_of(&s, "clear ready").unwrap();
    assert_eq!(
        handle_event(&mut s, Event::Click { row: r }),
        vec![Action::ClearReady("1".into())]
    );
}

// ----- scrolling -----

#[test]
fn scroll_moves_viewport_within_bounds() {
    let mut windows = Vec::new();
    for i in 0..40 {
        windows.push(win(&i.to_string(), "g", &format!("window-{i}")));
    }
    let mut s = State { windows, width: 30, height: 10, ..Default::default() };
    let max = s.max_scroll();
    assert!(max > 0);

    // Scroll up at the top stays at 0.
    handle_event(&mut s, Event::ScrollUp);
    assert_eq!(s.scroll, 0);

    // Scrolling down advances by 3 each wheel notch.
    handle_event(&mut s, Event::ScrollDown);
    assert_eq!(s.scroll, 3);
    handle_event(&mut s, Event::ScrollDown);
    assert_eq!(s.scroll, 6);

    // Scrolling down never exceeds max_scroll.
    for _ in 0..100 {
        handle_event(&mut s, Event::ScrollDown);
    }
    assert_eq!(s.scroll, max);

    // And back up clamps at 0.
    for _ in 0..100 {
        handle_event(&mut s, Event::ScrollUp);
    }
    assert_eq!(s.scroll, 0);
}

#[test]
fn clicking_while_scrolled_hits_the_right_window() {
    let mut windows = Vec::new();
    for i in 0..40 {
        windows.push(win(&i.to_string(), "g", &format!("window-{i}")));
    }
    let mut s = State { windows, width: 30, height: 10, ..Default::default() };
    s.scroll = 8;

    // Whatever window line sits at screen row 2 should be the one switched to.
    let lines = s.lines();
    let expected = match lines.get(s.scroll + 2).and_then(|l| l.target.clone()) {
        Some(Target::SwitchTo(i)) => Some(i),
        _ => None,
    };
    let actions = handle_event(&mut s, Event::Click { row: 2 });
    match expected {
        Some(i) => assert_eq!(actions, vec![Action::SwitchTo(i)]),
        None => assert!(actions.is_empty()),
    }
}

#[test]
fn q_key_quits() {
    let mut s = three_window_state();
    assert_eq!(handle_event(&mut s, Event::Key('q')), vec![Action::Quit]);
    assert!(handle_event(&mut s, Event::Key('x')).is_empty());
}

#[test]
fn stale_menu_clears_when_window_disappears() {
    let mut s = three_window_state();
    s.menu_window = "3".into();
    s.windows.retain(|w| w.index != "3");
    s.on_data_changed();
    assert_eq!(s.menu_window, "");
}
