//! `helyos login` / `helyos logout` — establish (and tear down) a pinned,
//! token-authenticated context for a remote daemon.

use std::io::IsTerminal;

use anyhow::{Context, Result};
use base64::Engine;

use crate::client::{self, HelyosClient};
use crate::config::{self, Context as CtxRecord};
use crate::output;

#[allow(clippy::struct_excessive_bools)]
pub struct LoginArgs<'a> {
    pub server: &'a str,
    pub name: Option<&'a str>,
    pub token: Option<&'a str>,
    pub token_stdin: bool,
    pub ca_file: Option<&'a str>,
    pub ca_fingerprint: Option<&'a str>,
    pub insecure: bool,
    pub project: Option<&'a str>,
    pub no_set_current: bool,
}

pub async fn login(args: LoginArgs<'_>) -> Result<()> {
    let server = normalize_server(args.server);
    let is_https = server.starts_with("https://");

    let mut ca_pem: Option<Vec<u8>> = None;
    if args.insecure {
        output::print_warning(
            "--insecure-skip-tls-verify: the server certificate will NOT be verified",
        );
    } else if is_https {
        if let Some(path) = args.ca_file {
            ca_pem = Some(std::fs::read(path).with_context(|| format!("read CA file {path}"))?);
        } else {
            ca_pem = Some(tofu_fetch_ca(&server, args.ca_fingerprint).await?);
        }
    }

    let token = acquire_token(args.token, args.token_stdin, &server)?;

    let probe = HelyosClient::new(&server, Some(&token), ca_pem.as_deref(), args.insecure);
    let who: serde_json::Value = probe
        .get("/api/v1/whoami")
        .await
        .context("could not validate the token (GET /api/v1/whoami failed)")?;
    let token_name = who["name"].as_str().map(str::to_string);

    let mut cfg = config::load();
    let ctx_name = match args.name {
        Some(n) => n.to_string(),
        None => unique_context_name(&cfg, &derive_name(&server)),
    };
    let ctx = CtxRecord {
        server: server.clone(),
        token: Some(token),
        ca: ca_pem
            .as_ref()
            .map(|p| base64::engine::general_purpose::STANDARD.encode(p)),
        ca_sha256: ca_pem.as_ref().map(|p| client::fingerprint_sha256(p)),
        project: args.project.map(str::to_string),
        insecure: args.insecure,
        token_name: token_name.clone(),
    };
    cfg.upsert(&ctx_name, ctx);
    if !args.no_set_current {
        cfg.set_current(&ctx_name)?;
    }
    cfg.save()?;

    let who_str = token_name.as_deref().unwrap_or("token");
    output::print_success(&format!(
        "Logged in to {server} as '{who_str}'. Context '{ctx_name}' is now active."
    ));
    Ok(())
}

pub fn logout(name: Option<&str>) -> Result<()> {
    let mut cfg = config::load();
    let target = match name {
        Some(n) => n.to_string(),
        None => cfg
            .active_name()
            .ok_or_else(|| anyhow::anyhow!("no active context to log out of"))?,
    };
    let ctx = cfg
        .contexts
        .get_mut(&target)
        .ok_or_else(|| anyhow::anyhow!("no context named '{target}'"))?;
    ctx.token = None;
    ctx.token_name = None;
    cfg.save()?;
    output::print_success(&format!(
        "Logged out of context '{target}' (server + pinned CA kept; re-login with one paste)"
    ));
    Ok(())
}

fn normalize_server(input: &str) -> String {
    let s = input.trim();
    let s = if s.starts_with("http://") || s.starts_with("https://") {
        s.to_string()
    } else {
        format!("https://{s}")
    };
    let after_scheme = s.split_once("://").map(|x| x.1).unwrap_or("");
    let authority = after_scheme.split('/').next().unwrap_or("");
    if authority.contains(':') {
        s
    } else {
        format!("{s}:6443")
    }
}

async fn tofu_fetch_ca(server: &str, expected_fp: Option<&str>) -> Result<Vec<u8>> {
    let boot = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .timeout(std::time::Duration::from_secs(10))
        .build()?;
    let resp = boot
        .get(format!("{server}/api/v1/ca"))
        .send()
        .await
        .context("fetch /api/v1/ca")?;
    if !resp.status().is_success() {
        anyhow::bail!(
            "server did not return a self-signed CA ({}). If it uses a publicly-trusted cert, \
             omit pinning; otherwise pass --ca-file <pem>.",
            resp.status()
        );
    }
    let body: serde_json::Value = resp.json().await.context("parse /api/v1/ca")?;
    let pem = body["pem"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("malformed /api/v1/ca response (no pem)"))?
        .as_bytes()
        .to_vec();
    let fp = client::fingerprint_sha256(&pem);

    match expected_fp {
        Some(expected) => {
            if !fingerprints_match(expected, &fp) {
                anyhow::bail!(
                    "CA fingerprint mismatch — aborting (possible MITM)\n  expected: {expected}\n  got:      {fp}"
                );
            }
        }
        None => {
            if !confirm_fingerprint(server, &fp)? {
                anyhow::bail!("aborted: CA not trusted");
            }
        }
    }
    Ok(pem)
}

fn acquire_token(flag: Option<&str>, stdin_flag: bool, server: &str) -> Result<String> {
    if let Some(t) = flag {
        return Ok(t.to_string());
    }
    if stdin_flag {
        let mut buf = String::new();
        std::io::stdin().read_line(&mut buf)?;
        return Ok(buf.trim().to_string());
    }
    if let Ok(t) = std::env::var("HELYOS_API_TOKEN") {
        if !t.is_empty() {
            return Ok(t);
        }
    }
    if std::io::stdin().is_terminal() {
        let t = dialoguer::Password::new()
            .with_prompt(format!("Paste a HELYOS API token for {server}"))
            .interact()?;
        return Ok(t);
    }
    anyhow::bail!("no token provided (use --token, --token-stdin, or HELYOS_API_TOKEN)")
}

fn confirm_fingerprint(server: &str, fp: &str) -> Result<bool> {
    if !std::io::stdin().is_terminal() {
        anyhow::bail!(
            "refusing to pin a CA non-interactively without --ca-fingerprint (server {server}, got {fp})"
        );
    }
    println!("The server {server} presented a self-signed CA:");
    println!("  SHA-256: {fp}");
    Ok(dialoguer::Confirm::new()
        .with_prompt("Trust and pin this CA?")
        .default(false)
        .interact()?)
}

fn fingerprints_match(a: &str, b: &str) -> bool {
    fn norm(s: &str) -> String {
        s.trim()
            .trim_start_matches("sha256:")
            .replace(':', "")
            .to_lowercase()
    }
    norm(a) == norm(b)
}

fn derive_name(server: &str) -> String {
    let host = server
        .split_once("://")
        .map(|x| x.1)
        .unwrap_or(server)
        .split('/')
        .next()
        .unwrap_or("")
        .split(':')
        .next()
        .unwrap_or("");
    let cleaned: String = host
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '-' { c } else { '-' })
        .collect();
    let cleaned = cleaned.trim_matches('-').to_string();
    if cleaned.is_empty() {
        "default".to_string()
    } else {
        cleaned
    }
}

fn unique_context_name(cfg: &config::Config, base: &str) -> String {
    if !cfg.contexts.contains_key(base) {
        return base.to_string();
    }
    let mut n = 2;
    loop {
        let candidate = format!("{base}-{n}");
        if !cfg.contexts.contains_key(&candidate) {
            return candidate;
        }
        n += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_adds_scheme_and_port() {
        assert_eq!(normalize_server("h.example"), "https://h.example:6443");
        assert_eq!(normalize_server("h.example:7000"), "https://h.example:7000");
        assert_eq!(normalize_server("https://h:6443"), "https://h:6443");
        assert_eq!(normalize_server("http://localhost:6443"), "http://localhost:6443");
    }

    #[test]
    fn fingerprints_match_normalizes() {
        assert!(fingerprints_match("9F:86:D0", "9f:86:d0"));
        assert!(fingerprints_match("sha256:9f86d0", "9f:86:d0"));
        assert!(!fingerprints_match("9f:86", "9f:87"));
    }

    #[test]
    fn derive_name_sanitizes_host() {
        assert_eq!(derive_name("https://helyos.acme.internal:6443"), "helyos-acme-internal");
        assert_eq!(derive_name("https://10.0.0.5:6443"), "10-0-0-5");
    }

    #[test]
    fn unique_name_suffixes_on_collision() {
        let cfg = config::Config {
            current_context: None,
            contexts: [("acme".to_string(), config::Context::default())].into_iter().collect(),
        };
        assert_eq!(unique_context_name(&cfg, "acme"), "acme-2");
        assert_eq!(unique_context_name(&cfg, "fresh"), "fresh");
    }
}
