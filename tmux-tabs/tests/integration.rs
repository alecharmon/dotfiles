//! Integration tests for the pure sidebar logic: grouping, layout, and the
//! mouse/scroll/click/key event reducer. No terminal or tmux involved.

use tmux_tabs::app::{handle_event, Action, Event, State};
use tmux_tabs::layout::{build_lines, LineStyle, Target};
use tmux_tabs::model::{
    adjacent_sidebar_window, clipped_text, grouped_windows, grouped_windows_with_sections,
    pr_refresh_targets, project_group, sidebar_window_order, ssh_host_from_command_line,
    window_display_name, Direction, PrState, PullRequestStatus, Window,
};
use tmux_tabs::sections::SectionLayout;

const HOME: &str = "/home/alec";
const CWD: &str = "/home/alec";

fn win(index: &str, group: &str, name: &str) -> Window {
    Window {
        index: index.to_string(),
        name: name.to_string(),
        group: group.to_string(),
        pane: format!("%{index}"),
        path: format!("/repo/{index}"),
        ..Default::default()
    }
}

// ----- model -----

#[test]
fn project_group_classifies_paths() {
    assert_eq!(project_group("/home/alec/dev", HOME, CWD), "dev");
    assert_eq!(project_group("/home/alec/dev/myrepo", HOME, CWD), "myrepo");
    assert_eq!(
        project_group("/home/alec/dev/myrepo/sub", HOME, CWD),
        "myrepo"
    );
    assert_eq!(project_group("/home/alec/dev/flora/api", HOME, CWD), "api");
    assert_eq!(
        project_group("/home/alec/dev/flora/api/x", HOME, CWD),
        "api"
    );
    assert_eq!(project_group("/home/alec/dev/flora", HOME, CWD), "flora");
    assert_eq!(
        project_group("/home/alec/worktrees/myrepo", HOME, CWD),
        "myrepo"
    );
    assert_eq!(
        project_group("/home/alec/worktrees/myrepo/sub", HOME, CWD),
        "myrepo"
    );
    assert_eq!(project_group("~/worktrees/zed", HOME, CWD), "zed");
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
    let names: Vec<String> = grouped_windows(&windows)
        .into_iter()
        .map(|(n, _)| n)
        .collect();
    assert_eq!(names, vec!["Apple", "zebra", "Other"]);
}

#[test]
fn custom_sections_appear_first_in_explicit_order() {
    let mut windows = vec![
        win("1", "zebra", "a"),
        win("2", "Apple", "b"),
        win("3", "Other", "c"),
        win("4", "Apple", "d"),
    ];
    windows[0].window_id = "@1".into();
    windows[1].window_id = "@2".into();
    windows[2].window_id = "@3".into();
    windows[3].window_id = "@4".into();
    let mut layout = SectionLayout::default();
    layout.add_window("Review", "@3");
    layout.add_window("Build", "@1");

    let groups: Vec<(String, Vec<String>)> = grouped_windows_with_sections(&windows, &layout)
        .into_iter()
        .map(|(name, ws)| (name, ws.into_iter().map(|w| w.index.clone()).collect()))
        .collect();

    assert_eq!(
        groups,
        vec![
            ("Review".to_string(), vec!["3".to_string()]),
            ("Build".to_string(), vec!["1".to_string()]),
            ("Apple".to_string(), vec!["2".to_string(), "4".to_string()]),
        ]
    );
}

#[test]
fn unassigned_windows_keep_folder_grouping() {
    let mut windows = vec![win("1", "zebra", "a"), win("2", "Apple", "b")];
    windows[0].window_id = "@1".into();
    windows[1].window_id = "@2".into();
    let mut layout = SectionLayout::default();
    layout.add_window("Review", "@1");

    let groups: Vec<(String, Vec<String>)> = grouped_windows_with_sections(&windows, &layout)
        .into_iter()
        .map(|(name, ws)| (name, ws.into_iter().map(|w| w.index.clone()).collect()))
        .collect();

    assert_eq!(
        groups,
        vec![
            ("Review".to_string(), vec!["1".to_string()]),
            ("Apple".to_string(), vec!["2".to_string()]),
        ]
    );
}

#[test]
fn folder_groups_render_with_folder_icon_but_custom_sections_do_not() {
    let mut windows = vec![win("1", "zebra", "a"), win("2", "Apple", "b")];
    windows[0].window_id = "@1".into();
    windows[1].window_id = "@2".into();
    let mut layout = SectionLayout::default();
    layout.add_window("🔎 Review", "@1");

    let group_text: Vec<String> =
        tmux_tabs::layout::build_lines_with_sections(&windows, &layout, "", 30)
            .into_iter()
            .filter_map(|line| match line.style {
                LineStyle::Group => Some(line.text),
                _ => None,
            })
            .collect();

    assert_eq!(group_text, vec!["🔎 Review", "📁 Apple"]);
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
    assert_eq!(
        adjacent_sidebar_window(&windows, "99", Direction::Next),
        "1"
    ); // unknown -> first
    assert_eq!(adjacent_sidebar_window(&[], "1", Direction::Next), "");
}

#[test]
fn pr_refresh_targets_include_only_windows_with_pane_and_path() {
    let mut valid = win("1", "Apple", "a");
    valid.pane = "%10".into();
    valid.path = "/repo/a".into();
    let mut no_pane = win("2", "Apple", "b");
    no_pane.pane.clear();
    no_pane.path = "/repo/b".into();
    let mut no_path = win("3", "Apple", "c");
    no_path.pane = "%30".into();
    no_path.path.clear();

    assert_eq!(
        pr_refresh_targets(&[valid, no_pane, no_path]),
        vec![("%10".to_string(), "/repo/a".to_string())]
    );
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
    assert_eq!(clipped_text("hello", 20), "hello");
    let clipped = clipped_text("abcdefghijklmnop", 10);
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
    let lines = build_lines(&s.windows, "", s.width);
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
    assert!(s.lines().iter().any(
        |l| matches!(&l.target, Some(Target::RunAction { action, .. }) if action == "rename")
    ));

    // Right-clicking the same window again closes it.
    handle_event(&mut s, Event::RightClick { row });
    assert_eq!(s.menu_window, "");
}

#[test]
fn action_menu_kill_kills_immediately() {
    let mut s = three_window_state();
    s.menu_window = "2".into();

    let kill_row = s
        .lines()
        .iter()
        .position(
            |l| matches!(&l.target, Some(Target::RunAction { action, .. }) if action == "kill"),
        )
        .unwrap() as u16;
    let actions = handle_event(&mut s, Event::Click { row: kill_row });
    assert_eq!(actions, vec![Action::KillWindow("2".into())]);
    assert_eq!(s.menu_window, "");
}

#[test]
fn action_menu_kill_with_worktree_emits_action() {
    let mut s = three_window_state();
    s.menu_window = "2".into();

    let r = row_of_action(&s, "kill + rm worktree").unwrap();
    assert_eq!(
        handle_event(&mut s, Event::Click { row: r }),
        vec![Action::KillWindowAndWorktree("2".into())]
    );
    assert_eq!(s.menu_window, "");
}

fn row_of_action(s: &State, action: &str) -> Option<u16> {
    s.lines()
        .iter()
        .position(|l| matches!(&l.target, Some(Target::RunAction { action: a, .. }) if a == action))
        .map(|p| p as u16)
}

#[test]
fn action_menu_rename_and_clear_ready_emit_actions() {
    let mut s = three_window_state();
    s.menu_window = "1".into();

    let r = row_of_action(&s, "rename").unwrap();
    assert_eq!(
        handle_event(&mut s, Event::Click { row: r }),
        vec![Action::Rename("1".into())]
    );
    assert_eq!(s.menu_window, "");

    s.menu_window = "1".into();
    let r = row_of_action(&s, "clear ready").unwrap();
    assert_eq!(
        handle_event(&mut s, Event::Click { row: r }),
        vec![Action::ClearReady("1".into())]
    );
}

#[test]
fn action_menu_always_offers_refresh_pr() {
    let mut s = three_window_state();
    s.menu_window = "1".into();

    assert!(row_of_action(&s, "refresh PR").is_some());
    assert!(row_of_action(&s, "open PR").is_none());
}

#[test]
fn action_menu_offers_open_pr_when_window_has_pr_url() {
    let mut s = three_window_state();
    s.windows[0].pr = Some(PullRequestStatus {
        number: 42,
        title: "Show PRs".into(),
        url: "https://github.com/example/repo/pull/42".into(),
        draft: false,
        state: PrState::Open,
    });
    s.menu_window = "1".into();

    assert!(row_of_action(&s, "open PR").is_some());
    assert!(row_of_action(&s, "refresh PR").is_some());
}

#[test]
fn layout_shows_open_pr_detail_line() {
    let mut s = three_window_state();
    s.windows[0].pr = Some(PullRequestStatus {
        number: 42,
        title: "Show PRs".into(),
        url: "https://github.com/example/repo/pull/42".into(),
        draft: false,
        state: PrState::Ready,
    });

    assert!(s.lines().iter().any(|l| l.text == "   ● PR #42 ready"));
    assert!(s.lines().iter().any(|l| matches!(
        l.style,
        LineStyle::Pr {
            state: PrState::Ready,
            ..
        }
    )));
}

#[test]
fn layout_marks_ci_running_pr_detail_line() {
    let mut s = three_window_state();
    s.windows[0].pr = Some(PullRequestStatus {
        number: 42,
        title: "Show PRs".into(),
        url: "https://github.com/example/repo/pull/42".into(),
        draft: false,
        state: PrState::CiRunning,
    });

    assert!(s.lines().iter().any(|l| l.text == "   ● PR #42 CI…"));
    assert!(s.lines().iter().any(|l| matches!(
        l.style,
        LineStyle::Pr {
            state: PrState::CiRunning,
            ..
        }
    )));
}

#[test]
fn layout_marks_failed_pr_detail_line() {
    let mut s = three_window_state();
    s.windows[0].pr = Some(PullRequestStatus {
        number: 42,
        title: "Show PRs".into(),
        url: "https://github.com/example/repo/pull/42".into(),
        draft: false,
        state: PrState::CiFailed,
    });

    assert!(s.lines().iter().any(|l| l.text == "   ● PR #42 failed"));
    assert!(s.lines().iter().any(|l| matches!(
        l.style,
        LineStyle::Pr {
            state: PrState::CiFailed,
            ..
        }
    )));
}

#[test]
fn layout_marks_draft_pr_detail_line() {
    let mut s = three_window_state();
    s.windows[0].pr = Some(PullRequestStatus {
        number: 42,
        title: "Show PRs".into(),
        url: "https://github.com/example/repo/pull/42".into(),
        draft: true,
        state: PrState::Draft,
    });

    assert!(s.lines().iter().any(|l| l.text == "   ● PR #42 draft"));
}

#[test]
fn single_click_on_pr_detail_line_switches_window() {
    let mut s = three_window_state();
    s.windows[0].pr = Some(PullRequestStatus {
        number: 42,
        title: "Show PRs".into(),
        url: "https://github.com/example/repo/pull/42".into(),
        draft: false,
        state: PrState::Ready,
    });

    let row = s
        .lines()
        .iter()
        .position(|l| l.text == "   ● PR #42 ready")
        .unwrap() as u16;

    assert_eq!(
        handle_event(&mut s, Event::Click { row }),
        vec![Action::SwitchTo("1".into())]
    );
}

#[test]
fn double_click_on_pr_detail_line_opens_pr() {
    let mut s = three_window_state();
    s.windows[0].pr = Some(PullRequestStatus {
        number: 42,
        title: "Show PRs".into(),
        url: "https://github.com/example/repo/pull/42".into(),
        draft: false,
        state: PrState::Ready,
    });

    let row = s
        .lines()
        .iter()
        .position(|l| l.text == "   ● PR #42 ready")
        .unwrap() as u16;

    assert_eq!(
        handle_event(&mut s, Event::DoubleClick { row }),
        vec![Action::OpenPr("1".into())]
    );
    // Double-clicking a non-PR row does nothing.
    assert!(handle_event(&mut s, Event::DoubleClick { row: 0 }).is_empty());
}

#[test]
fn clicking_pr_detail_without_url_switches_window() {
    let mut s = three_window_state();
    s.windows[0].pr = Some(PullRequestStatus {
        number: 42,
        title: "Show PRs".into(),
        url: String::new(),
        draft: false,
        state: PrState::Open,
    });

    let row = s
        .lines()
        .iter()
        .position(|l| l.text == "   ● PR #42 open")
        .unwrap() as u16;

    assert_eq!(
        handle_event(&mut s, Event::Click { row }),
        vec![Action::SwitchTo("1".into())]
    );
}

#[test]
fn right_clicking_pr_detail_opens_action_menu() {
    let mut s = three_window_state();
    s.windows[0].pr = Some(PullRequestStatus {
        number: 42,
        title: "Show PRs".into(),
        url: "https://github.com/example/repo/pull/42".into(),
        draft: false,
        state: PrState::Ready,
    });

    let row = s
        .lines()
        .iter()
        .position(|l| l.text == "   ● PR #42 ready")
        .unwrap() as u16;

    handle_event(&mut s, Event::RightClick { row });
    assert_eq!(s.menu_window, "1");
}

#[test]
fn action_menu_pr_items_emit_actions() {
    let mut s = three_window_state();
    s.windows[0].pr = Some(PullRequestStatus {
        number: 42,
        title: "Show PRs".into(),
        url: "https://github.com/example/repo/pull/42".into(),
        draft: false,
        state: PrState::Open,
    });
    s.menu_window = "1".into();

    let r = row_of_action(&s, "open PR").unwrap();
    assert_eq!(
        handle_event(&mut s, Event::Click { row: r }),
        vec![Action::OpenPr("1".into())]
    );
    assert_eq!(s.menu_window, "");

    s.menu_window = "1".into();
    let r = row_of_action(&s, "refresh PR").unwrap();
    assert_eq!(
        handle_event(&mut s, Event::Click { row: r }),
        vec![Action::RefreshPr("1".into())]
    );
    assert_eq!(s.menu_window, "");
}

// ----- scrolling -----

#[test]
fn scroll_moves_viewport_within_bounds() {
    let mut windows = Vec::new();
    for i in 0..40 {
        windows.push(win(&i.to_string(), "g", &format!("window-{i}")));
    }
    let mut s = State {
        windows,
        width: 30,
        height: 10,
        ..Default::default()
    };
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
    let mut s = State {
        windows,
        width: 30,
        height: 10,
        ..Default::default()
    };
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

#[test]
fn ssh_host_is_parsed_from_the_command_line() {
    assert_eq!(
        ssh_host_from_command_line("ssh box.example.com").as_deref(),
        Some("box.example.com")
    );
    assert_eq!(
        ssh_host_from_command_line("ssh -p 2222 -i ~/.ssh/id deploy@box.example.com uptime")
            .as_deref(),
        Some("box.example.com")
    );
    assert_eq!(
        ssh_host_from_command_line("/usr/bin/ssh -v ssh://root@10.0.0.4:22").as_deref(),
        Some("10.0.0.4")
    );
    // -p with an attached value must not swallow the destination.
    assert_eq!(
        ssh_host_from_command_line("ssh -p2222 devbox").as_deref(),
        Some("devbox")
    );
    assert_eq!(ssh_host_from_command_line("zsh -l"), None);
}

#[test]
fn ssh_windows_group_together_regardless_of_local_path() {
    let a = win("1", "ssh", "devbox");
    let b = win("2", "ssh", "prodbox");
    let local = win("3", "dotfiles", "vim");

    let windows = [a, b, local];
    let groups = grouped_windows(&windows);
    let ssh_group = groups.iter().find(|(n, _)| n == "ssh").expect("ssh group");

    assert_eq!(ssh_group.1.len(), 2);
    assert!(groups.iter().any(|(n, _)| n == "dotfiles"));
}

#[test]
fn herdr_group_is_pinned_to_the_top() {
    let mut layout = SectionLayout::default();
    layout.add_window("Review", "@7");

    let mut pinned = win("3", tmux_tabs::model::HERDR_GROUP, "herdr");
    let mut reviewed = win("1", "alpha", "a");
    reviewed.window_id = "@7".into();
    pinned.window_id = "@3".into();
    let windows = vec![reviewed, win("2", "beta", "b"), pinned];

    let names: Vec<String> = grouped_windows_with_sections(&windows, &layout)
        .into_iter()
        .map(|(n, _)| n)
        .collect();
    assert_eq!(names, vec!["herdr", "Review", "beta"]);
}
