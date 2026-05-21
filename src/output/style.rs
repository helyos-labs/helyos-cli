use console::Style;

pub fn print_success(msg: &str) {
    if super::is_json_mode() {
        return;
    }
    let style = Style::new().green();
    println!("{} {msg}", style.apply_to("✓"));
}

pub fn print_error(msg: &str) {
    if super::is_json_mode() {
        return;
    }
    let style = Style::new().red();
    eprintln!("{} {msg}", style.apply_to("✗"));
}

pub fn print_error_with_hint(msg: &str, hint: &str) {
    if super::is_json_mode() {
        return;
    }
    let err_style = Style::new().red();
    let hint_style = Style::new().dim();
    eprintln!("{} {msg}", err_style.apply_to("✗"));
    eprintln!("  {} {hint}", hint_style.apply_to("hint:"));
}

pub fn print_warning(msg: &str) {
    if super::is_json_mode() {
        return;
    }
    let style = Style::new().yellow();
    eprintln!("{} {msg}", style.apply_to("⚠"));
}

pub fn print_header(msg: &str) {
    if super::is_json_mode() {
        return;
    }
    let style = Style::new().bold();
    println!("{}", style.apply_to(msg));
}

pub fn print_kv(key: &str, value: &str) {
    if super::is_json_mode() {
        return;
    }
    let key_style = Style::new().bold();
    println!("{} {value}", key_style.apply_to(format!("{key}:")));
}
