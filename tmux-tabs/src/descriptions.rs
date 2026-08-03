//! Background terminal-description generation via `llm-redactor-exec` + `pi`.
//!
//! Mirrors the Python tool's behaviour: periodically capture each window's
//! content, and when it has changed meaningfully, ask the model for a one-line
//! summary and cache it. The sidebar reads the cache (see `tmux::tmux_windows`).
//! All of this is best-effort and silently disabled when the tools are absent.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::process::Command;
use std::time::Duration;

use crate::tmux::{active_pane, display, pane_var, tmux_windows, Ctx, RealTmux, Tmux};

fn which(bin: &str) -> bool {
    Command::new("sh")
        .arg("-c")
        .arg(format!("command -v {bin} >/dev/null 2>&1"))
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn normalize_capture(text: &str) -> String {
    let joined: String = text
        .lines()
        .map(|l| l.trim_end())
        .filter(|l| !l.trim().is_empty())
        .collect::<Vec<_>>()
        .join("\n");
    let chars: Vec<char> = joined.chars().collect();
    if chars.len() > 12000 {
        chars[chars.len() - 12000..].iter().collect()
    } else {
        joined
    }
}

fn signature(text: &str) -> String {
    let mut h = DefaultHasher::new();
    normalize_capture(text).hash(&mut h);
    format!("{:016x}", h.finish())
}

/// Cheap stand-in for difflib's ratio: similarity from the shared leading and
/// trailing characters, so appends/prepends score high (like SequenceMatcher).
fn similarity(a: &str, b: &str) -> f64 {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let total = a.len() + b.len();
    if total == 0 {
        return 1.0;
    }
    let min = a.len().min(b.len());
    let mut p = 0;
    while p < min && a[p] == b[p] {
        p += 1;
    }
    let mut s = 0;
    while s < min - p && a[a.len() - 1 - s] == b[b.len() - 1 - s] {
        s += 1;
    }
    let m = (p + s).min(min);
    (2.0 * m as f64) / total as f64
}

fn changed_significantly(old: &str, new: &str) -> bool {
    let old_n = normalize_capture(old);
    let new_n = normalize_capture(new);
    if old_n.is_empty() && !new_n.is_empty() {
        return true;
    }
    if new_n.is_empty() || old_n == new_n {
        return false;
    }
    let tail = |s: &str| -> String {
        let chars: Vec<char> = s.chars().collect();
        if chars.len() > 4000 {
            chars[chars.len() - 4000..].iter().collect()
        } else {
            s.to_string()
        }
    };
    similarity(&tail(&old_n), &tail(&new_n)) < 0.85
}

fn prompt(captured: &str) -> String {
    let norm = normalize_capture(captured);
    let tail: String = {
        let chars: Vec<char> = norm.chars().collect();
        if chars.len() > 8000 {
            chars[chars.len() - 8000..].iter().collect()
        } else {
            norm.clone()
        }
    };
    format!(
        "Describe what this terminal session is currently doing. \
Do not include secrets, keys, tokens, or private data. Output only compact JSON with two string fields: \
name (2-4 words, suitable for a tmux tab/window name) and description (at most 12 words).\n\n\
Terminal content:\n{tail}"
    )
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct GeneratedMetadata {
    name: String,
    description: String,
}

fn first_content_line(output: &str) -> String {
    for line in output.lines() {
        let line = line.trim();
        if line.is_empty()
            || line.starts_with('\u{1b}')
            || line.starts_with('╭')
            || line.starts_with('│')
            || line.starts_with('╰')
        {
            continue;
        }
        return line.chars().take(500).collect();
    }
    String::new()
}

fn clean_tab_name(name: &str) -> String {
    name.lines()
        .next()
        .unwrap_or("")
        .trim()
        .chars()
        .filter(|c| !c.is_control())
        .take(40)
        .collect()
}

fn clean_description(description: &str) -> String {
    description
        .lines()
        .next()
        .unwrap_or("")
        .trim()
        .chars()
        .take(200)
        .collect()
}

fn json_payload(output: &str) -> String {
    for line in output.lines() {
        let line = line.trim();
        if line.starts_with('{') && line.contains("\"description\"") {
            return line.to_string();
        }
    }
    first_content_line(output)
}

fn parse_generated_metadata(output: &str) -> GeneratedMetadata {
    let text = json_payload(output);
    let name = clean_tab_name(&json_field(&text, "name"));
    let description = clean_description(&json_field(&text, "description"));
    if !name.is_empty() || !description.is_empty() {
        return GeneratedMetadata { name, description };
    }
    GeneratedMetadata {
        name: String::new(),
        description: clean_description(&text),
    }
}

fn capture_window_text<T: Tmux>(t: &T, win_index: &str, home: &str) -> String {
    let pane = active_pane(t, win_index, home);
    if pane.is_empty() || pane_var(t, &pane, "#{pane_title}") == "tmux-tabs" {
        return String::new();
    }
    t.run(&["capture-pane", "-p", "-t", &pane, "-S", "-200"])
        .unwrap_or_default()
}

fn describe_with_pi(captured: &str) -> GeneratedMetadata {
    if !which("llm-redactor-exec") || !which("pi") {
        return GeneratedMetadata {
            name: String::new(),
            description: String::new(),
        };
    }
    let out = Command::new("llm-redactor-exec")
        .args([
            "--",
            "pi",
            "-p",
            "--no-session",
            "--model",
            "openai-codex/gpt-5.3-codex-spark",
            &prompt(captured),
        ])
        .output();
    match out {
        Ok(o) if o.status.success() => {
            parse_generated_metadata(&String::from_utf8_lossy(&o.stdout))
        }
        _ => GeneratedMetadata {
            name: String::new(),
            description: String::new(),
        },
    }
}

fn cache_path(ctx: &Ctx, session_id: &str, win_index: &str) -> std::path::PathBuf {
    ctx.descriptions_dir(session_id).join(format!(
        "{}-{}.json",
        session_id.trim_start_matches('$'),
        win_index
    ))
}

fn cached_record(ctx: &Ctx, session_id: &str, win_index: &str) -> (String, String) {
    let path = cache_path(ctx, session_id, win_index);
    let text = std::fs::read_to_string(path).unwrap_or_default();
    (json_field(&text, "signature"), json_field(&text, "sample"))
}

fn json_field(text: &str, key: &str) -> String {
    let needle = format!("\"{key}\"");
    let start = match text.find(&needle) {
        Some(i) => i + needle.len(),
        None => return String::new(),
    };
    let rest = &text[start..];
    let colon = match rest.find(':') {
        Some(i) => i,
        None => return String::new(),
    };
    let after = &rest[colon + 1..];
    let q1 = match after.find('"') {
        Some(i) => i,
        None => return String::new(),
    };
    let mut value = String::new();
    let mut chars = after[q1 + 1..].chars();
    while let Some(c) = chars.next() {
        match c {
            '"' => break,
            '\\' => {
                if let Some(n) = chars.next() {
                    match n {
                        'n' => value.push('\n'),
                        't' => value.push('\t'),
                        other => value.push(other),
                    }
                }
            }
            other => value.push(other),
        }
    }
    value
}

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

fn write_description(
    ctx: &Ctx,
    session_id: &str,
    win_index: &str,
    captured: &str,
    generated: &GeneratedMetadata,
) {
    let dir = ctx.descriptions_dir(session_id);
    if std::fs::create_dir_all(&dir).is_err() {
        return;
    }
    let sample: String = {
        let n = normalize_capture(captured);
        let chars: Vec<char> = n.chars().collect();
        if chars.len() > 4000 {
            chars[chars.len() - 4000..].iter().collect()
        } else {
            n
        }
    };
    let body = format!(
        "{{\n  \"name\": \"{}\",\n  \"description\": \"{}\",\n  \"sample\": \"{}\",\n  \"signature\": \"{}\",\n  \"window\": \"{}\"\n}}\n",
        json_escape(&generated.name),
        json_escape(&generated.description),
        json_escape(&sample),
        signature(captured),
        win_index
    );
    let _ = std::fs::write(cache_path(ctx, session_id, win_index), body);
}

fn rename_window<T: Tmux>(t: &T, win_index: &str, name: &str) {
    let name = clean_tab_name(name);
    if !name.is_empty() {
        let _ = t.run(&["rename-window", "-t", win_index, &name]);
    }
}

fn should_rename_window(_generated: &GeneratedMetadata) -> bool {
    false
}

/// One refresh pass over all windows. Returns true if any description changed.
pub fn refresh_once(ctx: &Ctx) -> bool {
    let t = RealTmux;
    let session_id = display(&t, "#{session_id}");
    let mut changed = false;
    for w in tmux_windows(&t, ctx) {
        let captured = capture_window_text(&t, &w.index, &ctx.home);
        if captured.trim().is_empty() {
            continue;
        }
        let (sig, sample) = cached_record(ctx, &session_id, &w.index);
        if sig == signature(&captured) {
            continue;
        }
        if !sample.is_empty() && !changed_significantly(&sample, &captured) {
            continue;
        }
        let generated = describe_with_pi(&captured);
        if !generated.description.is_empty() {
            write_description(ctx, &session_id, &w.index, &captured, &generated);
            if should_rename_window(&generated) {
                rename_window(&t, &w.index, &generated.name);
            }
            changed = true;
        }
    }
    changed
}

/// Spawn a background thread that refreshes descriptions on an interval.
pub fn spawn_refresher(ctx: Ctx) {
    if !which("llm-redactor-exec") || !which("pi") {
        return;
    }
    std::thread::spawn(move || loop {
        let _ = refresh_once(&ctx);
        std::thread::sleep(Duration::from_secs(60));
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn significance() {
        assert!(changed_significantly("", "hello world"));
        assert!(!changed_significantly("same\ntext", "same\ntext"));
        assert!(!changed_significantly("a\nb\nc\nd", "a\nb\nc\nd\ne"));
        assert!(changed_significantly("a\nb\nc\nd", "x\ny\nz\nw"));
    }

    #[test]
    fn json_roundtrip() {
        let escaped = json_escape("he said \"hi\"\nnext");
        let wrapped = format!("{{\"k\": \"{escaped}\"}}");
        assert_eq!(json_field(&wrapped, "k"), "he said \"hi\"\nnext");
    }

    #[test]
    fn parses_generated_name_and_description() {
        let generated = parse_generated_metadata(
            r#"{"name":"Fix tmux names","description":"adding automatic names to tmux tabs"}"#,
        );

        assert_eq!(generated.name, "Fix tmux names");
        assert_eq!(generated.description, "adding automatic names to tmux tabs");
    }

    #[test]
    fn parses_generated_metadata_from_fenced_output() {
        let generated = parse_generated_metadata(
            "```json\n{\"name\":\"Review PR\",\"description\":\"reviewing sidebar naming change\"}\n```",
        );

        assert_eq!(generated.name, "Review PR");
        assert_eq!(generated.description, "reviewing sidebar naming change");
    }

    #[test]
    fn generated_names_are_not_applied_to_tmux_windows() {
        let generated = GeneratedMetadata {
            name: "Review sidebar".to_string(),
            description: "reviewing sidebar description naming".to_string(),
        };

        assert!(!should_rename_window(&generated));
    }

    #[test]
    fn caches_generated_name_with_description() {
        let root = std::env::temp_dir().join(format!("tmux-tabs-desc-test-{}", std::process::id()));
        let ctx = Ctx {
            home: "/home/u".to_string(),
            cwd: "/home/u/dev/repo".to_string(),
            cache_home: root.clone(),
        };
        let generated = GeneratedMetadata {
            name: "Review sidebar".to_string(),
            description: "reviewing sidebar description naming".to_string(),
        };

        write_description(&ctx, "$1", "2", "terminal text", &generated);

        let text = std::fs::read_to_string(cache_path(&ctx, "$1", "2")).unwrap();
        assert_eq!(json_field(&text, "name"), "Review sidebar");
        assert_eq!(
            json_field(&text, "description"),
            "reviewing sidebar description naming"
        );
        let _ = std::fs::remove_dir_all(root);
    }
}
