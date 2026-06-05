//! `helyos context …` — manage named connection contexts (config-only; no server).

use anyhow::Result;

use crate::config::{self, Context};
use crate::output;

/// TLS mode label for display.
fn tls_mode(c: &Context) -> String {
    if c.server.starts_with("https://") {
        if c.insecure {
            "insecure".into()
        } else if c.ca.is_some() {
            "pinned".into()
        } else {
            "system".into()
        }
    } else {
        "http".into()
    }
}

pub fn list() -> Result<()> {
    let cfg = config::load();
    let active = cfg.active_name();

    if output::is_json_mode() {
        let arr: Vec<serde_json::Value> = cfg
            .contexts
            .iter()
            .map(|(n, c)| {
                serde_json::json!({
                    "name": n,
                    "server": c.server,
                    "project": c.project,
                    "tls": tls_mode(c),
                    "active": Some(n.as_str()) == active.as_deref(),
                })
            })
            .collect();
        output::print_json(&serde_json::json!(arr));
        return Ok(());
    }

    if cfg.contexts.is_empty() {
        println!("No contexts yet. Add one with: helyos login <server>");
        return Ok(());
    }

    let headers = ["", "name", "server", "project", "tls"];
    let rows: Vec<Vec<String>> = cfg
        .contexts
        .iter()
        .map(|(n, c)| {
            vec![
                if Some(n.as_str()) == active.as_deref() { "*".into() } else { " ".into() },
                n.clone(),
                c.server.clone(),
                c.project.clone().unwrap_or_else(|| "-".into()),
                tls_mode(c),
            ]
        })
        .collect();
    output::print_table(&headers, &rows);
    Ok(())
}

pub fn use_context(name: &str) -> Result<()> {
    let mut cfg = config::load();
    cfg.set_current(name)?;
    cfg.save()?;
    output::print_success(&format!("Switched to context '{name}'"));
    Ok(())
}

pub fn current() -> Result<()> {
    let cfg = config::load();
    let Some(name) = cfg.active_name() else {
        anyhow::bail!("no active context. Set one with: helyos context use <name>");
    };
    let c = &cfg.contexts[&name];
    if output::is_json_mode() {
        output::print_json(&serde_json::json!({
            "name": name, "server": c.server, "project": c.project, "tls": tls_mode(c),
        }));
        return Ok(());
    }
    println!("{name}");
    println!("  server:  {}", c.server);
    if let Some(p) = &c.project {
        println!("  project: {p}");
    }
    println!("  tls:     {}", tls_mode(c));
    Ok(())
}

pub fn remove(name: &str) -> Result<()> {
    let mut cfg = config::load();
    cfg.remove(name)?;
    cfg.save()?;
    output::print_success(&format!("Removed context '{name}'"));
    Ok(())
}

pub fn rename(old: &str, new: &str) -> Result<()> {
    let mut cfg = config::load();
    cfg.rename(old, new)?;
    cfg.save()?;
    output::print_success(&format!("Renamed context '{old}' → '{new}'"));
    Ok(())
}

pub fn set(name: &str, server: Option<&str>, project: Option<&str>) -> Result<()> {
    let mut cfg = config::load();
    let ctx = cfg
        .contexts
        .get_mut(name)
        .ok_or_else(|| anyhow::anyhow!("no context named '{name}'"))?;
    if let Some(s) = server {
        ctx.server = s.to_string();
    }
    if let Some(p) = project {
        ctx.project = Some(p.to_string());
    }
    cfg.save()?;
    output::print_success(&format!("Updated context '{name}'"));
    Ok(())
}
