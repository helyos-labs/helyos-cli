use console::Style;

// ---------------------------------------------------------------------------
// GitHub Dark color palette
// ---------------------------------------------------------------------------

/// Returns a `console::Style` for the given semantic color name.
///
/// Supported names: `bg`, `surface`, `border`, `border-subtle`,
/// `text`, `text-secondary`, `text-muted`,
/// `accent`, `green`, `yellow`, `red`, `cyan`.
pub fn color(name: &str) -> Style {
    let s = Style::new();
    match name {
        "bg" => s.color256(0),
        "surface" => s.color256(236),
        "border" => s.color256(239),
        "border-subtle" => s.color256(237),
        "text" => s.color256(253),
        "text-secondary" => s.color256(245),
        "text-muted" => s.color256(240),
        "accent" => s.color256(75),
        "green" => s.color256(77),
        "yellow" => s.color256(178),
        "red" => s.color256(203),
        "cyan" => s.color256(80),
        _ => s,
    }
}

// ---------------------------------------------------------------------------
// Icon system
// ---------------------------------------------------------------------------

/// Returns a terminal icon for the given semantic name.
///
/// By default, Unicode characters are used. Set `HELYOS_ICONS=nerd` in the
/// environment to use Nerd Font glyphs instead.
pub fn icon(name: &str) -> &'static str {
    let use_nerd = std::env::var("HELYOS_ICONS")
        .map(|v| v.to_lowercase() == "nerd")
        .unwrap_or(false);

    if use_nerd {
        match name {
            "cluster" => "\u{f1340}", // 󱍀
            "node" => "\u{f109}",     //
            "pod" => "\u{f111}",      //
            "deploy" => "\u{f013}",   //
            "event" => "\u{f0f3}",    //
            "success" => "\u{f00c}",  //
            "error" => "\u{f00d}",    //
            "warning" => "\u{f071}",  //
            "running" => "\u{f011}",  //
            _ => "?",
        }
    } else {
        match name {
            "cluster" => "⊞",
            "node" => "◆",
            "pod" => "●",
            "deploy" => "◎",
            "event" => "•",
            "success" => "✓",
            "error" => "✗",
            "warning" => "⚠",
            "running" => "⏻",
            _ => "?",
        }
    }
}

// ---------------------------------------------------------------------------
// Status helpers
// ---------------------------------------------------------------------------

/// Returns a colored `Style` for the given status string (case-insensitive).
pub fn status_style(status: &str) -> Style {
    match status.to_lowercase().as_str() {
        "running" => color("green"),
        "pending" | "creating" => color("cyan"),
        "restarting" | "degraded" => color("yellow"),
        "failed" | "crashloopbackoff" => color("red"),
        "stopped" | "stopping" => color("text-muted"),
        _ => color("text-secondary"),
    }
}

/// Returns a formatted `"● {status}"` string colored with `status_style`.
pub fn status_dot(status: &str) -> String {
    let style = status_style(status);
    let lower = status.to_lowercase();
    format!("{}", style.apply_to(format!("● {lower}")))
}

// ---------------------------------------------------------------------------
// Legacy print helpers (kept for backwards compatibility)
// ---------------------------------------------------------------------------

pub fn print_success(msg: &str) {
    if super::is_json_mode() {
        return;
    }
    println!("{} {msg}", color("green").apply_to(icon("success")));
}

pub fn print_error(msg: &str) {
    if super::is_json_mode() {
        return;
    }
    eprintln!("{} {msg}", color("red").apply_to(icon("error")));
}

pub fn print_error_with_hint(msg: &str, hint: &str) {
    if super::is_json_mode() {
        return;
    }
    let hint_style = color("text-muted");
    eprintln!("{} {msg}", color("red").apply_to(icon("error")));
    eprintln!("  {} {hint}", hint_style.apply_to("hint:"));
}

pub fn print_warning(msg: &str) {
    if super::is_json_mode() {
        return;
    }
    eprintln!("{} {msg}", color("yellow").apply_to(icon("warning")));
}

pub fn print_header(msg: &str) {
    if super::is_json_mode() {
        return;
    }
    println!("{}", color("text").bold().apply_to(msg));
}

pub fn print_kv(key: &str, value: &str) {
    if super::is_json_mode() {
        return;
    }
    let key_fmt = format!("{:>14}", key);
    println!(
        "{} {}",
        color("text-secondary").apply_to(key_fmt),
        color("text").apply_to(value)
    );
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn icon_returns_unicode_by_default() {
        // Ensure HELYOS_ICONS is not set to "nerd" for this test
        unsafe { std::env::remove_var("HELYOS_ICONS") };
        assert_eq!(icon("pod"), "●");
        assert_eq!(icon("success"), "✓");
        assert_eq!(icon("error"), "✗");
    }

    #[test]
    fn status_color_maps_correctly() {
        // Force ANSI colours on so styling is reflected in the formatted strings
        // even when stdout is not a TTY (e.g. during `cargo test`).
        console::set_colors_enabled(true);

        let green_str = format!("{}", color("green").apply_to("x"));
        let red_str = format!("{}", color("red").apply_to("x"));
        let yellow_str = format!("{}", color("yellow").apply_to("x"));

        assert_ne!(green_str, red_str);
        assert_ne!(green_str, yellow_str);
        assert_ne!(red_str, yellow_str);
    }

    #[test]
    fn status_style_returns_correct_color() {
        // Must not panic for known and unknown statuses
        let _running = status_style("running");
        let _unknown = status_style("unknown-xyz");
    }

    #[test]
    fn status_dot_formats_correctly() {
        let dot = status_dot("running");
        assert!(dot.contains("running"));
    }
}
