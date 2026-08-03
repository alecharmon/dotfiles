//! tmux-tabs — vertical window sidebar for tmux (Rust port).
//!
//! Modes:
//!   tmux-tabs                 toggle the sidebar (kill if open, else spawn)
//!   tmux-tabs --sidebar       run the sidebar TUI (invoked inside the split)
//!   tmux-tabs --select prev   move the sidebar to the previous grouped window
//!   tmux-tabs --select next   move the sidebar to the next grouped window

use tmux_tabs::agent::{
    add_section_window_by_ref, capture_tab, find_json, find_window, remove_section_window_by_ref,
    send_to_tab, windows_json,
};
use tmux_tabs::control::{select_adjacent_sidebar_window, toggle_sidebar};
use tmux_tabs::model::Direction;
use tmux_tabs::registry::{
    load as load_registry, now_string, registry_path, save as save_registry, TabRegistry, TabStatus,
};
use tmux_tabs::runtime;
use tmux_tabs::sections::{load as load_sections, save as save_sections};
use tmux_tabs::tmux::{display, tmux_windows, Ctx, RealTmux};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let ctx = Ctx::from_env();
    let t = RealTmux;

    if args.iter().any(|a| a == "--sidebar") {
        if let Err(e) = run_sidebar(ctx) {
            eprintln!("tmux-tabs: {e}");
            std::process::exit(1);
        }
    } else if args.iter().any(|a| a == "--list-json") {
        let (windows, _, _) = refresh_registry(&t, &ctx).unwrap_or_else(|_| {
            let windows = tmux_windows(&t, &ctx);
            (windows, TabRegistry::default(), registry_path(&ctx.home))
        });
        print!("{}", windows_json(&windows));
    } else if let Some(pos) = args.iter().position(|a| a == "--find") {
        let Some(reference) = args.get(pos + 1) else {
            eprintln!("tmux-tabs: --find requires a tab reference");
            std::process::exit(2);
        };
        let (windows, _, _) = refresh_registry(&t, &ctx).unwrap_or_else(|_| {
            let windows = tmux_windows(&t, &ctx);
            (windows, TabRegistry::default(), registry_path(&ctx.home))
        });
        print!("{}", find_json(&windows, reference));
    } else if let Some(pos) = args.iter().position(|a| a == "--capture") {
        let Some(reference) = args.get(pos + 1) else {
            eprintln!("tmux-tabs: --capture requires a tab reference");
            std::process::exit(2);
        };
        match capture_tab(&t, &ctx, reference) {
            Some(text) => println!("{text}"),
            None => {
                eprintln!("tmux-tabs: no tab matched {reference:?}");
                std::process::exit(1);
            }
        }
    } else if let Some(pos) = args.iter().position(|a| a == "--send") {
        let (Some(reference), Some(text)) = (args.get(pos + 1), args.get(pos + 2)) else {
            eprintln!("tmux-tabs: --send requires a tab reference and text");
            std::process::exit(2);
        };
        if !send_to_tab(&t, &ctx, reference, text, true) {
            eprintln!("tmux-tabs: no tab matched {reference:?}");
            std::process::exit(1);
        }
    } else if let Some(pos) = args.iter().position(|a| a == "--paste") {
        let (Some(reference), Some(text)) = (args.get(pos + 1), args.get(pos + 2)) else {
            eprintln!("tmux-tabs: --paste requires a tab reference and text");
            std::process::exit(2);
        };
        if !send_to_tab(&t, &ctx, reference, text, false) {
            eprintln!("tmux-tabs: no tab matched {reference:?}");
            std::process::exit(1);
        }
    } else if let Some(pos) = args.iter().position(|a| a == "--select") {
        let dir = match args.get(pos + 1).map(String::as_str) {
            Some("prev") => Direction::Prev,
            _ => Direction::Next,
        };
        select_adjacent_sidebar_window(&t, &ctx, dir);
    } else if let Some(pos) = args.iter().position(|a| a == "--section") {
        if let Err(e) = run_section_command(&t, &ctx, &args[pos + 1..]) {
            eprintln!("tmux-tabs: {e}");
            std::process::exit(1);
        }
    } else if let Some(pos) = args.iter().position(|a| a == "--tab") {
        if let Err(e) = run_tab_command(&t, &ctx, &args[pos + 1..]) {
            eprintln!("tmux-tabs: {e}");
            std::process::exit(1);
        }
    } else if args.iter().any(|a| a == "--tabs-json") {
        match refresh_registry(&t, &ctx) {
            Ok((_, registry, _)) => print!("{}", registry.to_json()),
            Err(e) => {
                eprintln!("tmux-tabs: {e}");
                std::process::exit(1);
            }
        }
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

fn refresh_registry(
    t: &RealTmux,
    ctx: &Ctx,
) -> Result<
    (
        Vec<tmux_tabs::model::Window>,
        TabRegistry,
        std::path::PathBuf,
    ),
    String,
> {
    let windows = tmux_windows(t, ctx);
    let session_id = display(t, "#{session_id}");
    let path = registry_path(&ctx.home);
    let mut registry = load_registry(&path);
    registry.refresh_live(&session_id, &windows, &now_string());
    save_registry(&path, &registry).map_err(|e| e.to_string())?;
    Ok((windows, registry, path))
}

fn tab_id_for_ref(windows: &[tmux_tabs::model::Window], reference: &str) -> Result<String, String> {
    let Some(window) = find_window(windows, reference) else {
        return Err(format!("no tab matched {reference:?}"));
    };
    if window.tab_id.is_empty() {
        return Err(format!("tab {reference:?} has no durable tab id"));
    }
    Ok(window.tab_id.clone())
}

fn run_tab_command(t: &RealTmux, ctx: &Ctx, args: &[String]) -> Result<(), String> {
    let Some(command) = args.first().map(String::as_str) else {
        return Err("--tab requires set-status|set-resume|status-json".to_string());
    };
    let (windows, mut registry, path) = refresh_registry(t, ctx)?;
    match command {
        "set-status" => {
            let (Some(reference), Some(status)) = (args.get(1), args.get(2)) else {
                return Err("--tab set-status requires a tab reference and status".to_string());
            };
            let status = TabStatus::parse(status).ok_or_else(|| {
                "status must be unknown|running|waiting_for_input|blocked|done|dead".to_string()
            })?;
            let tab_id = tab_id_for_ref(&windows, reference)?;
            if !registry.set_status(&tab_id, status) {
                return Err(format!("registry missing tab {tab_id}"));
            }
            save_registry(&path, &registry).map_err(|e| e.to_string())?;
        }
        "set-resume" => {
            let (Some(reference), Some(resume_command)) = (args.get(1), args.get(2)) else {
                return Err("--tab set-resume requires a tab reference and command".to_string());
            };
            let tab_id = tab_id_for_ref(&windows, reference)?;
            if !registry.set_resume(&tab_id, resume_command) {
                return Err(format!("registry missing tab {tab_id}"));
            }
            save_registry(&path, &registry).map_err(|e| e.to_string())?;
        }
        "status-json" => {
            let Some(reference) = args.get(1) else {
                return Err("--tab status-json requires a tab reference".to_string());
            };
            let tab_id = tab_id_for_ref(&windows, reference)?;
            match registry.find(&tab_id) {
                Some(record) => println!("{}", record.to_json()),
                None => println!("null"),
            }
        }
        _ => return Err("--tab requires set-status|set-resume|status-json".to_string()),
    }
    Ok(())
}

fn run_section_command(t: &RealTmux, ctx: &Ctx, args: &[String]) -> Result<(), String> {
    let Some(command) = args.first().map(String::as_str) else {
        return Err("--section requires create|add|remove|delete|list-json".to_string());
    };
    let session_id = display(t, "#{session_id}");
    let path = ctx.sections_path(&session_id);
    let mut layout = load_sections(&path);

    match command {
        "create" => {
            let Some(name) = args.get(1) else {
                return Err("--section create requires a name".to_string());
            };
            layout.create_section(name);
            save_sections(&path, &layout).map_err(|e| e.to_string())?;
        }
        "delete" => {
            let Some(name) = args.get(1) else {
                return Err("--section delete requires a name".to_string());
            };
            layout.delete_section(name);
            save_sections(&path, &layout).map_err(|e| e.to_string())?;
        }
        "add" => {
            let (Some(name), Some(reference)) = (args.get(1), args.get(2)) else {
                return Err("--section add requires a name and tab reference".to_string());
            };
            let windows = tmux_windows(t, ctx);
            if !add_section_window_by_ref(&mut layout, &windows, name, reference) {
                return Err(format!("no tab matched {reference:?}"));
            }
            let live: Vec<String> = windows.iter().map(|w| w.window_id.clone()).collect();
            layout.prune_missing(&live);
            save_sections(&path, &layout).map_err(|e| e.to_string())?;
        }
        "remove" => {
            let (Some(name), Some(reference)) = (args.get(1), args.get(2)) else {
                return Err("--section remove requires a name and tab reference".to_string());
            };
            let windows = tmux_windows(t, ctx);
            if !remove_section_window_by_ref(&mut layout, &windows, name, reference) {
                return Err(format!("no tab matched {reference:?}"));
            }
            save_sections(&path, &layout).map_err(|e| e.to_string())?;
        }
        "list-json" => {
            print!("{}", layout.to_json());
        }
        _ => return Err("--section requires create|add|remove|delete|list-json".to_string()),
    }
    Ok(())
}
