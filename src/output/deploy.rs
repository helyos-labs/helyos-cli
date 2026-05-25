use super::Panel;
use super::style;

pub struct DeployStep {
    pub icon: String,
    pub label: String,
    pub detail: String,
}

pub fn render_deploy_panel(name: &str, steps: &[DeployStep], status: &str, timing: &str) {
    if super::is_json_mode() {
        return;
    }

    let step_tuples: Vec<(&str, &str, &str)> = steps
        .iter()
        .map(|s| (s.icon.as_str(), s.label.as_str(), s.detail.as_str()))
        .collect();

    Panel::new(&format!("{} Deploying", style::icon("deploy")))
        .subtitle(name)
        .steps(&step_tuples)
        .footer_left(status)
        .footer_right(timing)
        .render();
}
