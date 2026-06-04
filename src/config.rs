//! Minimal, dependency-free config file support for nexa-cli.
//!
//! Loads `~/.nexa/config.toml` (override path with `NEXA_CONFIG`). The format is
//! a tiny subset of TOML: top-level `key = value` lines, optional double/single
//! quotes, `#` comments, blank lines, and `[section]` headers (ignored). Only
//! `server` and `token` are read. An absent or malformed file degrades
//! gracefully to an empty config rather than aborting the CLI.

use std::path::PathBuf;

/// Values loaded from the config file. Any field may be absent.
#[derive(Debug, Default, Clone)]
pub struct FileConfig {
    pub server: Option<String>,
    pub token: Option<String>,
}

/// Resolve the user's home directory without pulling in the `dirs` crate.
fn home_dir() -> Option<PathBuf> {
    // Unix/macOS: $HOME. Windows: $USERPROFILE.
    if let Some(h) = std::env::var_os("HOME") {
        if !h.is_empty() {
            return Some(PathBuf::from(h));
        }
    }
    if let Some(h) = std::env::var_os("USERPROFILE") {
        if !h.is_empty() {
            return Some(PathBuf::from(h));
        }
    }
    None
}

/// Path to the config file: `$NEXA_CONFIG` if set, else `~/.nexa/config.toml`.
pub fn config_path() -> Option<PathBuf> {
    if let Some(p) = std::env::var_os("NEXA_CONFIG") {
        if !p.is_empty() {
            return Some(PathBuf::from(p));
        }
    }
    home_dir().map(|h| h.join(".nexa").join("config.toml"))
}

/// Load config from the default path. Never fails: a missing file yields an
/// empty config; an unreadable file yields an empty config plus a stderr warning.
pub fn load() -> FileConfig {
    let Some(path) = config_path() else {
        return FileConfig::default();
    };
    match std::fs::read_to_string(&path) {
        Ok(contents) => parse(&contents),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => FileConfig::default(),
        Err(e) => {
            eprintln!(
                "Warning: could not read config file {}: {e}",
                path.display()
            );
            FileConfig::default()
        }
    }
}

/// Parse the minimal TOML-ish format. Unknown keys and malformed lines are
/// skipped silently (best-effort), so a partially broken file still yields any
/// keys it can.
fn parse(contents: &str) -> FileConfig {
    let mut cfg = FileConfig::default();
    for raw in contents.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with('[') {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let key = key.trim();
        let value = unquote(value.trim());
        if value.is_empty() {
            continue;
        }
        match key {
            "server" => cfg.server = Some(value),
            "token" => cfg.token = Some(value),
            _ => {}
        }
    }
    cfg
}

/// Strip surrounding double or single quotes, or any trailing `# comment`.
fn unquote(s: &str) -> String {
    let s = s.trim();
    if (s.starts_with('"') && s.ends_with('"') && s.len() >= 2)
        || (s.starts_with('\'') && s.ends_with('\'') && s.len() >= 2)
    {
        return s[1..s.len() - 1].to_string();
    }
    // Unquoted: drop an inline ` # comment` if present.
    match s.split_once(" #") {
        Some((before, _)) => before.trim().to_string(),
        None => s.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_quoted_and_unquoted() {
        let c = parse("server = \"http://h:6443\"\ntoken = abc123\n");
        assert_eq!(c.server.as_deref(), Some("http://h:6443"));
        assert_eq!(c.token.as_deref(), Some("abc123"));
    }

    #[test]
    fn ignores_comments_blanks_sections_and_unknown_keys() {
        let c = parse("# comment\n\n[default]\nserver = http://x:1 # inline\nfoo = bar\n");
        assert_eq!(c.server.as_deref(), Some("http://x:1"));
        assert_eq!(c.token, None);
    }

    #[test]
    fn single_quoted_value() {
        let c = parse("token = 'sekret'\n");
        assert_eq!(c.token.as_deref(), Some("sekret"));
    }

    #[test]
    fn malformed_lines_are_skipped() {
        let c = parse("this is not valid\ntoken=\nserver = ok\n");
        assert_eq!(c.server.as_deref(), Some("ok"));
        assert_eq!(c.token, None); // empty value skipped
    }

    #[test]
    fn empty_input_yields_default() {
        let c = parse("");
        assert!(c.server.is_none() && c.token.is_none());
    }
}
