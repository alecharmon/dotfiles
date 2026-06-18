//! tmux-tabs — vertical window sidebar for tmux (Rust port).
//!
//! Modes:
//!   tmux-tabs                 toggle the sidebar (kill if open, else spawn)
//!   tmux-tabs --sidebar       run the sidebar TUI (invoked inside the split)
//!   tmux-tabs --select prev   move the sidebar to the previous grouped window
//!   tmux-tabs --select next   move the sidebar to the next grouped window

use tmux_tabs::control::{select_adjacent_sidebar_window, toggle_sidebar};
use tmux_tabs::model::Direction;
use tmux_tabs::runtime;
use tmux_tabs::tmux::{Ctx, RealTmux};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let ctx = Ctx::from_env();
    let t = RealTmux;

    if args.iter().any(|a| a == "--sidebar") {
        if let Err(e) = run_sidebar(ctx) {
            eprintln!("tmux-tabs: {e}");
            std::process::exit(1);
        }
    } else if let Some(pos) = args.iter().position(|a| a == "--select") {
        let dir = match args.get(pos + 1).map(String::as_str) {
            Some("prev") => Direction::Prev,
            _ => Direction::Next,
        };
        select_adjacent_sidebar_window(&t, &ctx, dir);
    } else {
        let self_exe = std::env::current_exe()
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_else(|_| "tmux-tabs".to_string());
        toggle_sidebar(&t, &ctx, &self_exe);
    }
}

fn run_sidebar(ctx: Ctx) -> std::io::Result<()> {
    let mut rt = runtime::Runtime::new(ctx)?;
    rt.run()
}
