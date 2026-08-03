//! End-to-end tests: spawn a real, isolated tmux server, launch the actual
//! built binary as a sidebar pane, and drive it with real SGR mouse sequences
//! and key presses — exercising the full terminal → crossterm → tmux path.
//!
//! Mouse input is injected with `tmux send-keys -H <hex>`, writing the exact
//! bytes a terminal emits for a click/scroll into the sidebar pane's PTY, so
//! the binary's crossterm event parsing and handling run for real.

use std::process::Command;
use std::thread::sleep;
use std::time::{Duration, Instant};

const BIN: &str = env!("CARGO_BIN_EXE_tmux-tabs");

struct Server {
    sock: String,
    home: std::path::PathBuf,
}

impl Server {
    fn new(tag: &str) -> Self {
        let sock = format!("tmux-tabs-e2e-{}-{}", std::process::id(), tag);
        let home =
            std::env::temp_dir().join(format!("tmux-tabs-e2e-home-{}-{}", std::process::id(), tag));
        let _ = std::fs::create_dir_all(&home);
        let s = Server { sock, home };
        // Fresh server, predictable size, instant repaints.
        s.tmux(&[
            "new-session",
            "-d",
            "-s",
            "main",
            "-n",
            "alpha",
            "-x",
            "120",
            "-y",
            "40",
        ]);
        s.tmux(&["set-option", "-g", "status", "off"]);
        s.tmux(&["set-option", "-g", "escape-time", "0"]);
        s
    }

    fn raw(&self, args: &[&str]) -> std::process::Output {
        let mut full = vec!["-L", self.sock.as_str()];
        full.extend_from_slice(args);
        Command::new("tmux").args(&full).output().expect("run tmux")
    }

    fn tmux(&self, args: &[&str]) -> String {
        let out = self.raw(args);
        assert!(
            out.status.success(),
            "tmux {:?} failed: {}",
            args,
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8_lossy(&out.stdout)
            .trim_end_matches('\n')
            .to_string()
    }

    fn new_window(&self, name: &str) {
        self.tmux(&["new-window", "-n", name]);
    }

    fn active_window_name(&self) -> String {
        self.tmux(&["display-message", "-p", "#{window_name}"])
    }

    fn launch_sidebar(&self) {
        let home = self.home.to_string_lossy();
        let cmd = format!("HOME='{home}' XDG_CACHE_HOME='{home}/.cache' {BIN} --sidebar");
        // Spawn the sidebar to the left of the current (active) window.
        self.tmux(&["split-window", "-h", "-b", "-l", "26", cmd.as_str()]);
    }

    fn sidebar_pane(&self) -> Option<String> {
        let out = self.tmux(&["list-panes", "-a", "-F", "#{pane_id}|#{pane_title}"]);
        for line in out.lines() {
            let mut parts = line.splitn(2, '|');
            let id = parts.next().unwrap_or("");
            let title = parts.next().unwrap_or("");
            if title == "tmux-tabs" {
                return Some(id.to_string());
            }
        }
        None
    }

    fn capture(&self, pane: &str) -> Vec<String> {
        self.tmux(&["capture-pane", "-p", "-t", pane])
            .lines()
            .map(|l| l.to_string())
            .collect()
    }

    /// Wait until the sidebar pane exists and its capture contains `needle`.
    fn wait_for_render(&self, needle: &str) -> String {
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            if let Some(pane) = self.sidebar_pane() {
                let body = self.capture(&pane).join("\n");
                if body.contains(needle) {
                    return pane;
                }
            }
            if Instant::now() > deadline {
                let dump = self
                    .sidebar_pane()
                    .map(|p| self.capture(&p).join("\n"))
                    .unwrap_or_else(|| "<no sidebar pane>".into());
                panic!("sidebar did not render '{needle}' in time. Saw:\n{dump}");
            }
            sleep(Duration::from_millis(100));
        }
    }

    /// Index (0-based screen row) of the first captured line containing `needle`.
    fn row_of(&self, pane: &str, needle: &str) -> Option<u32> {
        self.capture(pane)
            .iter()
            .position(|l| l.contains(needle))
            .map(|p| p as u32)
    }

    /// Send raw bytes to the pane's input as hex (tmux send-keys -H).
    fn send_bytes(&self, pane: &str, bytes: &[u8]) {
        let mut args: Vec<String> = vec!["send-keys".into(), "-t".into(), pane.into(), "-H".into()];
        for b in bytes {
            args.push(format!("{:02x}", b));
        }
        let refs: Vec<&str> = args.iter().map(String::as_str).collect();
        self.tmux(&refs);
    }

    /// Inject an SGR mouse event. Coordinates are 1-based (terminal convention).
    fn mouse(&self, pane: &str, button: u32, col: u32, row: u32, press: bool) {
        let final_byte = if press { 'M' } else { 'm' };
        let seq = format!("\x1b[<{button};{col};{row}{final_byte}");
        self.send_bytes(pane, seq.as_bytes());
    }

    fn left_click(&self, pane: &str, row_0based: u32) {
        let row = row_0based + 1;
        self.mouse(pane, 0, 3, row, true);
        self.mouse(pane, 0, 3, row, false);
    }

    fn right_click(&self, pane: &str, row_0based: u32) {
        let row = row_0based + 1;
        self.mouse(pane, 2, 3, row, true);
        self.mouse(pane, 2, 3, row, false);
    }

    fn scroll_down(&self, pane: &str, notches: u32) {
        for _ in 0..notches {
            self.mouse(pane, 65, 3, 5, true); // SGR button 65 == wheel down
        }
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.raw(&["kill-server"]);
        let _ = std::fs::remove_dir_all(&self.home);
    }
}

fn settle() {
    // Give the binary's 200ms poll loop time to read input and repaint.
    sleep(Duration::from_millis(500));
}

#[test]
fn sidebar_launches_renders_and_quits() {
    let s = Server::new("launch");
    s.new_window("bravo");
    s.new_window("charlie");
    s.launch_sidebar();

    let pane = s.wait_for_render("charlie");
    let body = s.capture(&pane).join("\n");
    assert!(body.contains("alpha"), "missing alpha:\n{body}");
    assert!(body.contains("bravo"), "missing bravo:\n{body}");

    // Press 'q' to quit; the sidebar pane should disappear.
    s.send_bytes(&pane, b"q");
    let deadline = Instant::now() + Duration::from_secs(5);
    while s.sidebar_pane().is_some() {
        if Instant::now() > deadline {
            panic!("sidebar pane still present after 'q'");
        }
        sleep(Duration::from_millis(100));
    }
}

#[test]
fn mouse_click_switches_window() {
    let s = Server::new("click");
    s.new_window("bravo");
    s.new_window("charlie"); // active window after creation
    s.launch_sidebar();

    let pane = s.wait_for_render("alpha");
    assert_eq!(s.active_window_name(), "charlie");

    // Click the "alpha" row in the sidebar.
    let row = s.row_of(&pane, "alpha").expect("alpha row visible");
    s.left_click(&pane, row);
    settle();

    assert_eq!(
        s.active_window_name(),
        "alpha",
        "left click should switch the active window to alpha"
    );
}

#[test]
fn mouse_scroll_moves_viewport() {
    let s = Server::new("scroll");
    // Enough windows to overflow a 40-row pane (each window takes ~2 lines).
    for i in 0..30 {
        s.new_window(&format!("win{i:02}"));
    }
    s.launch_sidebar();

    let pane = s.wait_for_render("win00");
    let top_before = s.capture(&pane).into_iter().find(|l| !l.trim().is_empty());

    s.scroll_down(&pane, 6);
    settle();

    let top_after = s.capture(&pane).into_iter().find(|l| !l.trim().is_empty());
    assert_ne!(
        top_before, top_after,
        "wheel-scroll should change the first visible line"
    );
}

#[test]
fn right_click_opens_action_menu() {
    let s = Server::new("menu");
    s.new_window("bravo");
    s.launch_sidebar();

    let pane = s.wait_for_render("alpha");
    let row = s.row_of(&pane, "bravo").expect("bravo row visible");
    s.right_click(&pane, row);
    settle();

    let body = s.capture(&pane).join("\n");
    assert!(body.contains("rename"), "menu missing 'rename':\n{body}");
    assert!(body.contains("kill"), "menu missing 'kill':\n{body}");
    assert!(
        body.contains("clear ready"),
        "menu missing 'clear ready':\n{body}"
    );
}
