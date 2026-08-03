//! Persistent custom sidebar sections.
//!
//! Agents can create named sections and assign tmux windows to them. The CLI
//! accepts friendly tab references, but this module stores stable tmux
//! `window_id`s so assignments survive window renumbering.

use std::fs;
use std::io;
use std::path::Path;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SectionLayout {
    pub sections: Vec<Section>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Section {
    pub name: String,
    pub windows: Vec<String>,
}

impl SectionLayout {
    pub fn create_section(&mut self, name: &str) {
        let name = name.trim();
        if name.is_empty() || self.sections.iter().any(|s| s.name == name) {
            return;
        }
        self.sections.push(Section {
            name: name.to_string(),
            windows: Vec::new(),
        });
    }

    pub fn delete_section(&mut self, name: &str) {
        self.sections.retain(|s| s.name != name);
    }

    pub fn add_window(&mut self, section_name: &str, window_id: &str) {
        let section_name = section_name.trim();
        let window_id = window_id.trim();
        if section_name.is_empty() || window_id.is_empty() {
            return;
        }
        for section in &mut self.sections {
            section.windows.retain(|id| id != window_id);
        }
        self.create_section(section_name);
        if let Some(section) = self.sections.iter_mut().find(|s| s.name == section_name) {
            section.windows.push(window_id.to_string());
        }
    }

    pub fn remove_window(&mut self, section_name: &str, window_id: &str) {
        if let Some(section) = self.sections.iter_mut().find(|s| s.name == section_name) {
            section.windows.retain(|id| id != window_id);
        }
    }

    pub fn prune_missing(&mut self, live_window_ids: &[String]) {
        for section in &mut self.sections {
            section.windows.retain(|id| live_window_ids.contains(id));
        }
    }

    pub fn to_json(&self) -> String {
        let mut out = String::from("{\n  \"sections\": [");
        for (i, section) in self.sections.iter().enumerate() {
            if i > 0 {
                out.push(',');
            }
            out.push_str("\n    {\n");
            out.push_str(&format!(
                "      \"name\": \"{}\",\n",
                json_escape(&section.name)
            ));
            out.push_str("      \"windows\": [");
            for (j, window) in section.windows.iter().enumerate() {
                if j > 0 {
                    out.push_str(", ");
                }
                out.push_str(&format!("\"{}\"", json_escape(window)));
            }
            out.push_str("]\n    }");
        }
        if !self.sections.is_empty() {
            out.push('\n');
            out.push_str("  ");
        }
        out.push_str("]\n}\n");
        out
    }

    pub fn from_json(text: &str) -> Self {
        let mut layout = SectionLayout::default();
        let mut rest = text;
        while let Some(name_pos) = rest.find("\"name\"") {
            rest = &rest[name_pos + "\"name\"".len()..];
            let Some(name) = json_string_after_colon(rest) else {
                break;
            };
            let Some(windows_pos) = rest.find("\"windows\"") else {
                break;
            };
            rest = &rest[windows_pos + "\"windows\"".len()..];
            let windows = json_string_array_after_colon(rest);
            layout.sections.push(Section { name, windows });
        }
        layout
    }
}

pub fn load(path: &Path) -> SectionLayout {
    fs::read_to_string(path)
        .map(|text| SectionLayout::from_json(&text))
        .unwrap_or_default()
}

pub fn save(path: &Path, layout: &SectionLayout) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, layout.to_json())
}

fn json_escape(s: &str) -> String {
    let mut out = String::new();
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

fn json_string_after_colon(text: &str) -> Option<String> {
    let colon = text.find(':')?;
    let after = &text[colon + 1..];
    let q1 = after.find('"')?;
    let after_q = &after[q1 + 1..];
    parse_json_string(after_q).map(|(s, _)| s)
}

fn json_string_array_after_colon(text: &str) -> Vec<String> {
    let Some(colon) = text.find(':') else {
        return Vec::new();
    };
    let after = &text[colon + 1..];
    let Some(open) = after.find('[') else {
        return Vec::new();
    };
    let Some(close) = after[open + 1..].find(']') else {
        return Vec::new();
    };
    let mut values = Vec::new();
    let mut body = &after[open + 1..open + 1 + close];
    while let Some(q) = body.find('"') {
        let after_q = &body[q + 1..];
        let Some((value, used)) = parse_json_string(after_q) else {
            break;
        };
        values.push(value);
        body = &after_q[used..];
    }
    values
}

fn parse_json_string(text_after_open_quote: &str) -> Option<(String, usize)> {
    let mut value = String::new();
    let mut escaped = false;
    for (i, c) in text_after_open_quote.char_indices() {
        if escaped {
            value.push(match c {
                'n' => '\n',
                't' => '\t',
                other => other,
            });
            escaped = false;
        } else if c == '\\' {
            escaped = true;
        } else if c == '"' {
            return Some((value, i + 1));
        } else {
            value.push(c);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_delete_and_roundtrip_sections() {
        let mut layout = SectionLayout::default();
        layout.create_section("Review");
        layout.create_section("Review");
        layout.create_section("Build");

        assert_eq!(layout.sections.len(), 2);
        assert_eq!(layout.sections[0].name, "Review");

        let decoded = SectionLayout::from_json(&layout.to_json());
        assert_eq!(decoded, layout);

        layout.delete_section("Review");
        assert_eq!(
            layout.sections,
            vec![Section {
                name: "Build".into(),
                windows: vec![]
            }]
        );
    }

    #[test]
    fn add_window_moves_it_out_of_other_sections() {
        let mut layout = SectionLayout::default();
        layout.add_window("Review", "@7");
        layout.add_window("Tests", "@7");
        layout.add_window("Tests", "@9");

        assert_eq!(
            layout.sections[0],
            Section {
                name: "Review".into(),
                windows: vec![]
            }
        );
        assert_eq!(
            layout.sections[1],
            Section {
                name: "Tests".into(),
                windows: vec!["@7".into(), "@9".into()]
            }
        );
    }

    #[test]
    fn remove_window_only_affects_named_section() {
        let mut layout = SectionLayout::default();
        layout.add_window("Review", "@7");
        layout.add_window("Review", "@9");
        layout.remove_window("Review", "@7");

        assert_eq!(layout.sections[0].windows, vec!["@9".to_string()]);
    }

    #[test]
    fn prune_missing_removes_dead_window_ids() {
        let mut layout = SectionLayout::default();
        layout.add_window("Review", "@7");
        layout.add_window("Review", "@9");
        layout.prune_missing(&["@9".to_string()]);

        assert_eq!(layout.sections[0].windows, vec!["@9".to_string()]);
    }
}
