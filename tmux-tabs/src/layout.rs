//! Turns the window list + menu state into a flat list of visual lines.
//!
//! Each line carries an optional click `Target`, so mapping a mouse click at
//! screen row `y` to an action is just `lines[scroll + y].target`. Keeping this
//! pure makes scroll/click behaviour directly testable without a terminal.

use crate::model::{
    action_menu_labels, clipped_text, grouped_windows_with_sections, window_icon, PrState, Window,
    DETAIL_INDENT, HERDR_GROUP,
};
use crate::sections::SectionLayout;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Target {
    SwitchTo(String),
    OpenPr(String),
    RunAction { index: String, action: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineStyle {
    Blank,
    Group,
    WindowName {
        active: bool,
        bell: bool,
    },
    Detail {
        active: bool,
        bell: bool,
    },
    Pr {
        active: bool,
        bell: bool,
        state: PrState,
    },
    Action,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VisualLine {
    pub text: String,
    pub style: LineStyle,
    pub target: Option<Target>,
}

impl VisualLine {
    fn blank() -> Self {
        VisualLine {
            text: String::new(),
            style: LineStyle::Blank,
            target: None,
        }
    }
}

/// Build the full visual-line layout for the sidebar.
pub fn build_lines(windows: &[Window], menu_window: &str, width: usize) -> Vec<VisualLine> {
    build_lines_with_sections(windows, &SectionLayout::default(), menu_window, width)
}

/// Build the full visual-line layout, applying persistent custom sections.
pub fn build_lines_with_sections(
    windows: &[Window],
    sections: &SectionLayout,
    menu_window: &str,
    width: usize,
) -> Vec<VisualLine> {
    let w = width;
    let mut lines: Vec<VisualLine> = Vec::new();
    lines.push(VisualLine::blank()); // top padding

    let grouped = grouped_windows_with_sections(windows, sections);

    for (gi, (group, group_windows)) in grouped.into_iter().enumerate() {
        if gi > 0 {
            lines.push(VisualLine::blank());
        }
        let group_text = if group == HERDR_GROUP {
            format!("🐑 {group}")
        } else if sections.sections.iter().any(|s| s.name == group) {
            group.clone()
        } else {
            format!("📁 {group}")
        };
        lines.push(VisualLine {
            text: group_text,
            style: LineStyle::Group,
            target: None,
        });

        for win in group_windows {
            let active = win.active;
            let bell = win.bell;
            let target = Target::SwitchTo(win.index.clone());

            let icon = window_icon(win);
            lines.push(VisualLine {
                text: format!("{icon}{}", win.name),
                style: LineStyle::WindowName { active, bell },
                target: Some(target.clone()),
            });

            let title = clipped_text(win.title.trim_start_matches('✳'), w);
            if !title.is_empty() {
                lines.push(VisualLine {
                    text: format!("{DETAIL_INDENT}{title}"),
                    style: LineStyle::Detail { active, bell },
                    target: Some(target.clone()),
                });
            }
            if let Some(pr) = &win.pr {
                let pr_target = if pr.url.is_empty() {
                    target.clone()
                } else {
                    Target::OpenPr(win.index.clone())
                };
                lines.push(VisualLine {
                    text: format!("{DETAIL_INDENT}● PR #{} {}", pr.number, pr.state.label()),
                    style: LineStyle::Pr {
                        active,
                        bell,
                        state: pr.state,
                    },
                    target: Some(pr_target),
                });
            }

            lines.push(VisualLine::blank()); // margin below window

            if menu_window == win.index {
                let has_pr_url = win
                    .pr
                    .as_ref()
                    .map(|pr| !pr.url.is_empty())
                    .unwrap_or(false);
                for action in action_menu_labels(has_pr_url) {
                    lines.push(VisualLine {
                        text: format!("   {action}"),
                        style: LineStyle::Action,
                        target: Some(Target::RunAction {
                            index: win.index.clone(),
                            action: action.to_string(),
                        }),
                    });
                }
            }
        }
    }

    lines
}

/// Resolve the click target at visual-line offset `idx` (already includes the
/// scroll offset), if any.
pub fn target_at(lines: &[VisualLine], idx: usize) -> Option<&Target> {
    lines.get(idx).and_then(|l| l.target.as_ref())
}
