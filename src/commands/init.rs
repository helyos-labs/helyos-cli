use anyhow::Result;
use std::fs;
use std::path::Path;

use crate::output;

pub fn generate_template(project: &str, deployment: &str, image: &str) -> String {
    format!(
        r#"# Helyos deployment spec — {deployment}
# Docs: https://github.com/helyos-labs/helyos

project: {project}

deployment:
  name: {deployment}

image: {image}
replicas: 1

# ports:
#   - 3000

# env:
#   DATABASE_URL: "postgres://localhost/mydb"

# network:
#   public: true
#   domain: {deployment}.example.com
#   https: true

# healthcheck:
#   path: /health
#   interval: 10s
#   timeout: 5s
#   retries: 3

# volumes:
#   - name: data
#     mount: /app/data

# restart: always  # always | onfailure | never
"#
    )
}

pub fn init(name: Option<&str>, image: Option<&str>) -> Result<()> {
    let (project, deployment, image) = if let Some(name) = name {
        let img = image.unwrap_or("nginx:alpine");
        (name.to_string(), name.to_string(), img.to_string())
    } else {
        use dialoguer::{Input, theme::ColorfulTheme};

        let project: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Project name")
            .interact_text()?;

        let deployment: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Deployment name")
            .default(project.clone())
            .interact_text()?;

        let img: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Container image")
            .default("nginx:alpine".into())
            .interact_text()?;

        (project, deployment, img)
    };

    let dir = Path::new(&project);
    if dir.join("app.yaml").exists() {
        anyhow::bail!(
            "{}/app.yaml already exists — use a different name or remove the existing file",
            project
        );
    }

    fs::create_dir_all(dir)?;
    let template = generate_template(&project, &deployment, &image);
    fs::write(dir.join("app.yaml"), &template)?;

    if output::is_json_mode() {
        output::print_json(&serde_json::json!({
            "project": project,
            "deployment": deployment,
            "image": image,
            "file": format!("{}/app.yaml", project),
        }));
        return Ok(());
    }

    output::print_success(&format!("Created {}/app.yaml", project));
    println!();
    println!("Next steps:");
    println!(
        "  1. Edit {}/app.yaml to customize your deployment",
        project
    );
    println!("  2. Deploy with: helyos deploy {}/app.yaml", project);
    println!();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn template_contains_project_name() {
        let t = generate_template("myapp", "myapp", "nginx:alpine");
        assert!(t.contains("project: myapp"));
        assert!(t.contains("name: myapp"));
        assert!(t.contains("image: nginx:alpine"));
    }

    #[test]
    fn template_has_commented_sections() {
        let t = generate_template("myapp", "api", "node:20");
        assert!(t.contains("# ports:"));
        assert!(t.contains("# env:"));
        assert!(t.contains("# network:"));
        assert!(t.contains("# healthcheck:"));
        assert!(t.contains("# volumes:"));
    }

    #[test]
    fn template_is_valid_yaml_for_uncommented_lines() {
        let t = generate_template("test", "api", "nginx:latest");
        let uncommented: String = t
            .lines()
            .filter(|l| !l.trim_start().starts_with('#') && !l.is_empty())
            .collect::<Vec<_>>()
            .join("\n");
        let value: serde_yaml_ng::Value = serde_yaml_ng::from_str(&uncommented).unwrap();
        assert_eq!(
            value["project"],
            serde_yaml_ng::Value::String("test".into())
        );
    }

    /// Uncomment every example section (everything but the doc-header comments)
    /// and feed it to the real deployment parser — exactly what a user gets when
    /// they uncomment a block in the scaffold. Catches example fields whose names
    /// the parser does not accept.
    fn uncomment_examples(template: &str) -> String {
        template
            .lines()
            .filter(|l| !l.starts_with("# Helyos") && !l.starts_with("# Docs"))
            .map(|l| {
                if let Some(rest) = l.strip_prefix("# ") {
                    rest.to_string()
                } else if l == "#" {
                    String::new()
                } else {
                    l.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[test]
    fn uncommented_template_parses_as_valid_spec() {
        let template = generate_template("shop", "api", "nginx:alpine");
        let yaml = uncomment_examples(&template);

        let spec = helyos_core::config::parse_deployment(&yaml)
            .expect("uncommented scaffold template must parse as a valid deployment spec");

        assert_eq!(spec.volumes.len(), 1);
        assert_eq!(spec.volumes[0].mount_point(), "/app/data");
    }

    #[test]
    fn restart_legend_lists_only_real_policy_values() {
        use helyos_core::domain::models::RestartPolicy;

        let template = generate_template("shop", "api", "nginx:alpine");
        let line = template
            .lines()
            .find(|l| l.contains("restart:"))
            .expect("template documents a restart policy");

        // The legend is the inline `# always | … | …` after the example value.
        let legend = line.rsplit('#').next().unwrap();
        let values: Vec<&str> = legend
            .split('|')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .collect();
        assert!(
            !values.is_empty(),
            "restart legend should list policy values"
        );

        for v in values {
            let parsed: std::result::Result<RestartPolicy, _> = serde_yaml_ng::from_str(v);
            assert!(
                parsed.is_ok(),
                "restart legend lists `{v}`, which is not a valid RestartPolicy value"
            );
        }
    }
}
