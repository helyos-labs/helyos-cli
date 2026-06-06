//! Config file support for helyos-cli: named, switchable contexts.
//!
//! Loads/saves `~/.helyos/config.toml` (override path with `HELYOS_CONFIG`).
//! Format is a tiny TOML subset: top-level `key = value`, `[context.NAME]`
//! section headers, `#` comments, blank lines. A legacy file with only
//! top-level `server`/`token` (no `[context.*]` sections) is read as an
//! implicit context named "default" so existing setups keep working.

use std::collections::BTreeMap;
use std::path::PathBuf;

/// One named target: where + who + defaults. `server` is required; the rest are
/// optional. `ca`/`ca_sha256`/`insecure`/`token_name` are populated by
/// `helyos login` (M4) and are treated as opaque pass-through values here.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Context {
    pub server: String,
    pub token: Option<String>,
    pub ca: Option<String>,
    pub ca_sha256: Option<String>,
    pub project: Option<String>,
    pub insecure: bool,
    pub token_name: Option<String>,
}

/// The whole config: a set of named contexts + the active one.
#[derive(Debug, Default, Clone)]
pub struct Config {
    pub current_context: Option<String>,
    pub contexts: BTreeMap<String, Context>,
}

fn home_dir() -> Option<PathBuf> {
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

/// Path to the config file: `$HELYOS_CONFIG` if set, else `~/.helyos/config.toml`.
pub fn config_path() -> Option<PathBuf> {
    if let Some(p) = std::env::var_os("HELYOS_CONFIG") {
        if !p.is_empty() {
            return Some(PathBuf::from(p));
        }
    }
    home_dir().map(|h| h.join(".helyos").join("config.toml"))
}

/// Load config from the default path. Never fails: a missing file yields an
/// empty config; an unreadable file yields an empty config plus a warning.
pub fn load() -> Config {
    let Some(path) = config_path() else {
        return Config::default();
    };
    match std::fs::read_to_string(&path) {
        Ok(contents) => parse(&contents),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Config::default(),
        Err(e) => {
            eprintln!("Warning: could not read config file {}: {e}", path.display());
            Config::default()
        }
    }
}

/// Strip surrounding double/single quotes, or a trailing ` # comment`.
fn unquote(s: &str) -> String {
    let s = s.trim();
    if (s.starts_with('"') && s.ends_with('"') && s.len() >= 2)
        || (s.starts_with('\'') && s.ends_with('\'') && s.len() >= 2)
    {
        return s[1..s.len() - 1].to_string();
    }
    match s.split_once(" #") {
        Some((before, _)) => before.trim().to_string(),
        None => s.to_string(),
    }
}

/// Parse the TOML-subset: top-level `current-context`, legacy top-level
/// `server`/`token` (→ a "default" context), and `[context.NAME]` sections.
fn parse(contents: &str) -> Config {
    let mut cfg = Config::default();
    let mut legacy = Context::default();
    let mut legacy_seen = false;
    // None = top-level; Some("") = inside an unknown section (ignore); Some(name) = a context.
    let mut current: Option<String> = None;

    for raw in contents.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if let Some(section) = line.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
            let section = section.trim();
            if let Some(name) = section.strip_prefix("context.") {
                let name = unquote(name);
                if !name.is_empty() {
                    cfg.contexts.entry(name.clone()).or_default();
                    current = Some(name);
                    continue;
                }
            }
            current = Some(String::new()); // unknown section → ignore its keys
            continue;
        }

        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let key = key.trim();
        let value = unquote(value.trim());

        match &current {
            None => match key {
                "current-context" if !value.is_empty() => {
                    cfg.current_context = Some(value);
                }
                "server" => {
                    legacy.server = value;
                    legacy_seen = true;
                }
                "token" if !value.is_empty() => {
                    legacy.token = Some(value);
                    legacy_seen = true;
                }
                _ => {}
            },
            Some(name) if name.is_empty() => {}
            Some(name) => {
                let ctx = cfg.contexts.entry(name.clone()).or_default();
                apply_key(ctx, key, &value);
            }
        }
    }

    // Legacy flat file (no [context.*] sections) → synthesize a "default" context.
    if legacy_seen && (!legacy.server.is_empty() || legacy.token.is_some()) && cfg.contexts.is_empty() {
        cfg.contexts.insert("default".to_string(), legacy);
    }
    cfg
}

fn apply_key(ctx: &mut Context, key: &str, value: &str) {
    if value.is_empty() && key != "insecure" {
        return;
    }
    match key {
        "server" => ctx.server = value.to_string(),
        "token" => ctx.token = Some(value.to_string()),
        "ca" => ctx.ca = Some(value.to_string()),
        "ca-sha256" => ctx.ca_sha256 = Some(value.to_string()),
        "project" => ctx.project = Some(value.to_string()),
        "token-name" => ctx.token_name = Some(value.to_string()),
        "insecure" => ctx.insecure = value == "true",
        _ => {}
    }
}

impl Config {
    /// The active context: `current-context` if set & present, else the sole
    /// context when there is exactly one, else `None`.
    pub fn active(&self) -> Option<&Context> {
        if let Some(name) = &self.current_context {
            if let Some(c) = self.contexts.get(name) {
                return Some(c);
            }
        }
        if self.contexts.len() == 1 {
            return self.contexts.values().next();
        }
        None
    }

    /// Name of the active context (same rules as [`active`]).
    pub fn active_name(&self) -> Option<String> {
        if let Some(name) = &self.current_context {
            if self.contexts.contains_key(name) {
                return Some(name.clone());
            }
        }
        if self.contexts.len() == 1 {
            return self.contexts.keys().next().cloned();
        }
        None
    }

    /// Set the active context. Errors if `name` is unknown.
    pub fn set_current(&mut self, name: &str) -> anyhow::Result<()> {
        if !self.contexts.contains_key(name) {
            anyhow::bail!("no context named '{name}'");
        }
        self.current_context = Some(name.to_string());
        Ok(())
    }

    /// Insert or replace a context.
    pub fn upsert(&mut self, name: &str, ctx: Context) {
        self.contexts.insert(name.to_string(), ctx);
    }

    /// Remove a context. Clears `current-context` if it pointed there.
    pub fn remove(&mut self, name: &str) -> anyhow::Result<()> {
        if self.contexts.remove(name).is_none() {
            anyhow::bail!("no context named '{name}'");
        }
        if self.current_context.as_deref() == Some(name) {
            self.current_context = None;
        }
        Ok(())
    }

    /// Rename a context (and follow `current-context`). Errors if `new` exists
    /// or `old` is unknown.
    pub fn rename(&mut self, old: &str, new: &str) -> anyhow::Result<()> {
        if self.contexts.contains_key(new) {
            anyhow::bail!("context '{new}' already exists");
        }
        let ctx = self
            .contexts
            .remove(old)
            .ok_or_else(|| anyhow::anyhow!("no context named '{old}'"))?;
        self.contexts.insert(new.to_string(), ctx);
        if self.current_context.as_deref() == Some(old) {
            self.current_context = Some(new.to_string());
        }
        Ok(())
    }

    /// Render the config to the TOML-subset text (known keys only).
    pub fn serialize(&self) -> String {
        let mut out =
            String::from("# Managed by `helyos`. Edit with `helyos context` / `helyos login`.\n");
        if let Some(cc) = &self.current_context {
            out.push_str(&format!("current-context = {}\n", quote(cc)));
        }
        for (name, c) in &self.contexts {
            out.push_str(&format!("\n[context.{name}]\n"));
            out.push_str(&format!("server = {}\n", quote(&c.server)));
            if let Some(t) = &c.token {
                out.push_str(&format!("token = {}\n", quote(t)));
            }
            if let Some(p) = &c.project {
                out.push_str(&format!("project = {}\n", quote(p)));
            }
            if let Some(tn) = &c.token_name {
                out.push_str(&format!("token-name = {}\n", quote(tn)));
            }
            if c.insecure {
                out.push_str("insecure = true\n");
            }
            if let Some(ca) = &c.ca {
                out.push_str(&format!("ca = {}\n", quote(ca)));
            }
            if let Some(s) = &c.ca_sha256 {
                out.push_str(&format!("ca-sha256 = {}\n", quote(s)));
            }
        }
        out
    }

    /// Save to the default config path (`config_path()`).
    pub fn save(&self) -> anyhow::Result<()> {
        let path = config_path().ok_or_else(|| anyhow::anyhow!("could not determine config path"))?;
        self.save_to(&path)
    }

    /// Save to a specific path: back up any existing file to `*.toml.bak`, then
    /// write atomically (temp file in the same dir + rename) with `0600` perms.
    pub fn save_to(&self, path: &std::path::Path) -> anyhow::Result<()> {
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)?;
            }
        }
        if path.exists() {
            let _ = std::fs::copy(path, path.with_extension("toml.bak"));
        }
        let tmp = path.with_extension("toml.tmp");
        std::fs::write(&tmp, self.serialize().as_bytes())?;
        set_owner_only(&tmp)?;
        std::fs::rename(&tmp, path)?;
        Ok(())
    }
}

/// Quote a value for the TOML-subset. Config values (URLs, tokens, base64,
/// project names) never contain `"`; we wrap in double quotes without escaping
/// to match the simple `unquote` reader.
fn quote(s: &str) -> String {
    format!("\"{s}\"")
}

#[cfg(unix)]
fn set_owner_only(path: &std::path::Path) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))
}

#[cfg(not(unix))]
fn set_owner_only(_path: &std::path::Path) -> std::io::Result<()> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_named_contexts_and_current() {
        let c = parse(
            "current-context = \"prod\"\n\n[context.local]\nserver = \"http://localhost:6443\"\ntoken = abc\nproject = default\n\n[context.prod]\nserver = \"https://h:6443\"\ninsecure = true\n",
        );
        assert_eq!(c.current_context.as_deref(), Some("prod"));
        assert_eq!(c.contexts.len(), 2);
        let local = &c.contexts["local"];
        assert_eq!(local.server, "http://localhost:6443");
        assert_eq!(local.token.as_deref(), Some("abc"));
        assert_eq!(local.project.as_deref(), Some("default"));
        assert!(c.contexts["prod"].insecure);
    }

    #[test]
    fn legacy_flat_file_becomes_default_context() {
        let c = parse("server = http://localhost:6443\ntoken = sekret\n");
        assert_eq!(c.contexts.len(), 1);
        let d = &c.contexts["default"];
        assert_eq!(d.server, "http://localhost:6443");
        assert_eq!(d.token.as_deref(), Some("sekret"));
    }

    #[test]
    fn active_prefers_current_then_single() {
        let two = parse("[context.a]\nserver = http://a\n[context.b]\nserver = http://b\n");
        assert!(two.active().is_none(), "ambiguous with 2 contexts and no current-context");
        let cur = parse("current-context = a\n[context.a]\nserver = http://a\n[context.b]\nserver = http://b\n");
        assert_eq!(cur.active().unwrap().server, "http://a");
        let one = parse("[context.only]\nserver = http://only\n");
        assert_eq!(one.active().unwrap().server, "http://only");
        assert_eq!(one.active_name().as_deref(), Some("only"));
    }

    #[test]
    fn unknown_sections_and_keys_ignored() {
        let c = parse("[weird]\nfoo = bar\n[context.x]\nserver = http://x\nbogus = 1\n");
        assert_eq!(c.contexts.len(), 1);
        assert_eq!(c.contexts["x"].server, "http://x");
    }

    #[test]
    fn empty_input_is_empty_config() {
        let c = parse("");
        assert!(c.contexts.is_empty() && c.current_context.is_none());
        assert!(c.active().is_none());
    }

    #[test]
    fn serialize_then_parse_roundtrips() {
        let mut cfg = Config::default();
        cfg.contexts.insert(
            "prod".into(),
            Context {
                server: "https://h:6443".into(),
                token: Some("nxa-api_abc".into()),
                project: Some("web".into()),
                insecure: true,
                token_name: Some("alice".into()),
                ca: Some("BASE64CA".into()),
                ca_sha256: Some("9f:86".into()),
            },
        );
        cfg.current_context = Some("prod".into());

        let round = parse(&cfg.serialize());
        assert_eq!(round.current_context.as_deref(), Some("prod"));
        assert_eq!(round.contexts["prod"], cfg.contexts["prod"]);
    }

    #[test]
    fn save_to_writes_file_and_backup() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        std::fs::write(&path, "current-context = \"old\"\n[context.old]\nserver = http://old\n").unwrap();

        let mut cfg = Config::default();
        cfg.contexts.insert("new".into(), Context { server: "http://new".into(), ..Default::default() });
        cfg.current_context = Some("new".into());
        cfg.save_to(&path).unwrap();

        let reloaded = parse(&std::fs::read_to_string(&path).unwrap());
        assert_eq!(reloaded.current_context.as_deref(), Some("new"));
        assert!(reloaded.contexts.contains_key("new"));
        let bak = std::fs::read_to_string(path.with_extension("toml.bak")).unwrap();
        assert!(bak.contains("context.old"), "backup must retain the prior file");
    }

    #[cfg(unix)]
    #[test]
    fn save_to_sets_owner_only_perms() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        Config::default().save_to(&path).unwrap();
        let mode = std::fs::metadata(&path).unwrap().permissions().mode();
        assert_eq!(mode & 0o777, 0o600);
    }

    #[test]
    fn mutators_set_current_remove_rename() {
        let mut cfg = parse("[context.a]\nserver = http://a\n[context.b]\nserver = http://b\n");
        assert!(cfg.set_current("a").is_ok());
        assert!(cfg.set_current("missing").is_err());
        cfg.rename("a", "a2").unwrap();
        assert_eq!(cfg.current_context.as_deref(), Some("a2"));
        cfg.remove("a2").unwrap();
        assert!(cfg.current_context.is_none(), "removing the active context clears current");
        assert!(cfg.remove("a2").is_err());
    }
}
