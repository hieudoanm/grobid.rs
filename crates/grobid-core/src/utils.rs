/// Escape a string for safe inclusion in double-quoted YAML.
pub fn yaml_string(value: &str) -> String {
    let escaped = value.replace('\\', "\\\\").replace('"', "\\\"");
    format!("\"{}\"", escaped)
}

use std::sync::Mutex;

/// A simple timestamped logger with severity levels.
pub struct Logger {
    level: u8,
}

const LEVELS: &[(u8, &str)] = &[
    (0, "fatal"),
    (1, "error"),
    (2, "warn"),
    (3, "info"),
    (4, "debug"),
    (5, "trace"),
];

impl Logger {
    pub fn new(level: &str) -> Self {
        let level = LEVELS
            .iter()
            .find(|(_, name)| name.eq_ignore_ascii_case(level))
            .map(|(l, _)| *l)
            .unwrap_or(3);
        Logger { level }
    }

    fn log(&self, level: u8, message: &str) {
        if level > self.level {
            return;
        }
        let name = LEVELS
            .iter()
            .find(|(l, _)| *l == level)
            .map(|(_, name)| name.to_uppercase())
            .unwrap_or_else(|| "INFO".to_string());
        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
        let line = format!("[{timestamp}] [{name}] {message}");
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

    pub fn warning(&self, message: &str) {
        self.warn(message);
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
    &LOGGER
}

static LOGGER: std::sync::LazyLock<Logger> = std::sync::LazyLock::new(|| Logger::new("info"));

/// Format a message with positional arguments (printf-style).
pub fn format_msg(message: &str, args: &[&str]) -> String {
    let mut result = message.to_string();
    for arg in args {
        if let Some(pos) = result.find("{}") {
            result.replace_range(pos..pos + 2, arg);
        } else {
            break;
        }
    }
    result
}

/// Normalize text: lowercase, replace non-alphanumeric runs with a single space.
pub fn normalize(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut prev_space = false;
    for ch in text.chars() {
        if ch.is_alphanumeric() {
            result.push(ch.to_ascii_lowercase());
            prev_space = false;
        } else if !prev_space {
            result.push(' ');
            prev_space = true;
        }
    }
    result.trim().to_string()
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

/// A small helper to hold a static `Mutex<Option<Logger>>` for reconfiguration.
pub fn set_level(level: &str) {
    let mut guard = LOG_LEVEL.lock().unwrap();
    *guard = level.to_string();
}

static LOG_LEVEL: Mutex<String> = Mutex::new(String::new());
