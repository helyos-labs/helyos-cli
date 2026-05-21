mod age;
#[allow(dead_code)]
mod spinner;
#[allow(dead_code)]
mod style;
mod table;

pub use age::format_age;
pub use spinner::Spinner;
#[allow(unused_imports)]
pub use style::{print_error, print_error_with_hint, print_header, print_kv, print_success, print_warning};
pub use table::print_table;

use std::sync::atomic::{AtomicBool, Ordering};

static JSON_MODE: AtomicBool = AtomicBool::new(false);

pub fn set_json_mode(enabled: bool) {
    JSON_MODE.store(enabled, Ordering::Relaxed);
}

pub fn is_json_mode() -> bool {
    JSON_MODE.load(Ordering::Relaxed)
}

pub fn print_json(value: &impl serde::Serialize) {
    println!("{}", serde_json::to_string_pretty(value).unwrap());
}
