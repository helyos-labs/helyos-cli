use console::Style;
use indicatif::{ProgressBar, ProgressStyle};

pub struct Spinner {
    bar: ProgressBar,
}

impl Spinner {
    pub fn new(message: &str) -> Self {
        let bar = ProgressBar::new_spinner();
        bar.set_style(
            ProgressStyle::default_spinner()
                .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
                .template("{spinner} {msg}")
                .expect("invalid spinner template"),
        );
        bar.set_message(message.to_string());
        bar.enable_steady_tick(std::time::Duration::from_millis(80));
        Self { bar }
    }

    pub fn set_message(&self, message: &str) {
        self.bar.set_message(message.to_string());
    }

    pub fn finish_success(&self, message: &str) {
        let style = Style::new().green();
        self.bar
            .finish_with_message(format!("{} {message}", style.apply_to("✓")));
    }

    pub fn finish_error(&self, message: &str) {
        let style = Style::new().red();
        self.bar
            .finish_with_message(format!("{} {message}", style.apply_to("✗")));
    }

    pub fn finish_clear(&self) {
        self.bar.finish_and_clear();
    }
}
