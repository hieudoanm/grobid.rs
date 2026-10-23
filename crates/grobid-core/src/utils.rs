//! Logging and text helpers.

use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::LazyLock;

/// Escape a string for safe inclusion in double-quoted YAML.
pub fn yaml_string(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len() + 2);
    for ch in value.chars() {
        match ch {
            '\\' => escaped.push_str("\\\\"),
            '"' => escaped.push_str("\\\""),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            c if c.is_control() => escaped.push_str(&format!("\\u{:04X}", c as u32)),
            c => escaped.push(c),
        }
    }
    format!("\"{}\"", escaped)
}

const LEVELS: &[(u8, &str)] = &[
    (0, "fatal"),
    (1, "error"),
    (2, "warn"),
    (3, "info"),
    (4, "debug"),
    (5, "trace"),
];

fn parse_level(name: &str) -> Option<u8> {
    LEVELS
        .iter()
        .find(|(_, n)| n.eq_ignore_ascii_case(name))
        .map(|(l, _)| *l)
}

/// Default level from `GROBID_LOG` (e.g. `debug`, `warn`), falling back to info.
static DEFAULT_LEVEL: LazyLock<u8> = LazyLock::new(|| {
    std::env::var("GROBID_LOG")
        .ok()
        .and_then(|v| parse_level(&v))
        .unwrap_or(3)
});

/// Runtime override set by CLI flags; `u8::MAX` means "use the default".
static OVERRIDE: AtomicU8 = AtomicU8::new(u8::MAX);

/// Set the log level by name (`fatal`..`trace`); unknown names are ignored.
pub fn set_level(name: &str) {
    if let Some(level) = parse_level(name) {
        OVERRIDE.store(level, Ordering::Relaxed);
    }
}

/// `-v`/`--verbose` occurrences: 1 → debug, 2+ → trace. Overrides `GROBID_LOG`.
pub fn bump_level(count: u8) {
    let level = (3 + count).min(5);
    OVERRIDE.store(level, Ordering::Relaxed);
}

fn level_name(level: u8) -> String {
    LEVELS
        .iter()
        .find(|(l, _)| *l == level)
        .map(|(_, name)| name.to_uppercase())
        .unwrap_or_else(|| "INFO".to_string())
}

/// A simple timestamped logger with severity levels.
pub struct Logger;

impl Logger {
    fn log(&self, level: u8, message: &str) {
        let override_level = OVERRIDE.load(Ordering::Relaxed);
        let threshold = if override_level == u8::MAX {
            *DEFAULT_LEVEL
        } else {
            override_level
        };
        if level > threshold {
            return;
        }
        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
        let line = format!("[{timestamp}] [{}] {message}", level_name(level));
        if level <= 1 {
            eprintln!("{line}");
        } else {
            println!("{line}");
        }
    }

    pub fn fatal(&self, message: &str) -> ! {
        self.log(0, message);
        std::process::exit(1);
    }

    pub fn error(&self, message: &str) {
        self.log(1, message);
    }

    pub fn warn(&self, message: &str) {
        self.log(2, message);
    }

    pub fn info(&self, message: &str) {
        self.log(3, message);
    }

    pub fn debug(&self, message: &str) {
        self.log(4, message);
    }

    pub fn trace(&self, message: &str) {
        self.log(5, message);
    }
}

/// Global logger instance.
pub fn log() -> &'static Logger {
    static LOGGER: Logger = Logger;
    &LOGGER
}

/// Concatenate all text descendants of an element, collapsing whitespace.
pub fn text_content(element: &roxmltree::Node) -> String {
    let mut parts: Vec<String> = Vec::new();
    collect_text(element, &mut parts);
    let joined = parts.join(" ");
    collapse_whitespace(&joined).trim().to_string()
}

fn collect_text(node: &roxmltree::Node, out: &mut Vec<String>) {
    if node.is_text() {
        out.push(node.text().unwrap_or("").to_string());
    }
    for child in node.children() {
        collect_text(&child, out);
    }
}

pub fn collapse_whitespace(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut prev_space = false;
    for ch in s.chars() {
        if ch.is_whitespace() {
            if !prev_space {
                result.push(' ');
                prev_space = true;
            }
        } else {
            result.push(ch);
            prev_space = false;
        }
    }
    result
}
