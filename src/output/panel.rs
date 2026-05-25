use super::style;

pub struct Panel {
    title: String,
    count: Option<String>,
    subtitle: Option<String>,
    content: PanelContent,
    footer_left: Option<String>,
    footer_right: Option<String>,
}

enum PanelContent {
    None,
    Table {
        headers: Vec<String>,
        rows: Vec<Vec<String>>,
    },
    Kv(Vec<(String, String)>),
    Steps(Vec<(String, String, String)>),
}

impl Panel {
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            count: None,
            subtitle: None,
            content: PanelContent::None,
            footer_left: None,
            footer_right: None,
        }
    }

    pub fn count(mut self, count: &str) -> Self {
        self.count = Some(count.to_string());
        self
    }

    pub fn subtitle(mut self, sub: &str) -> Self {
        self.subtitle = Some(sub.to_string());
        self
    }

    pub fn table(mut self, headers: &[&str], rows: &[Vec<String>]) -> Self {
        self.content = PanelContent::Table {
            headers: headers.iter().map(|h| h.to_string()).collect(),
            rows: rows.to_vec(),
        };
        self
    }

    pub fn kv(mut self, pairs: &[(&str, &str)]) -> Self {
        self.content = PanelContent::Kv(
            pairs
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
        );
        self
    }

    pub fn steps(mut self, steps: &[(&str, &str, &str)]) -> Self {
        self.content = PanelContent::Steps(
            steps
                .iter()
                .map(|(icon, label, desc)| (icon.to_string(), label.to_string(), desc.to_string()))
                .collect(),
        );
        self
    }

    pub fn footer_left(mut self, text: &str) -> Self {
        self.footer_left = Some(text.to_string());
        self
    }

    pub fn footer_right(mut self, text: &str) -> Self {
        self.footer_right = Some(text.to_string());
        self
    }

    #[allow(dead_code)]
    pub fn render(&self) {
        if super::is_json_mode() {
            return;
        }
        let width = terminal_width();
        let output = self.build(width);
        print!("{output}");
    }

    #[allow(dead_code)]
    pub fn render_to_string(&self) -> String {
        self.build(terminal_width())
    }

    #[allow(dead_code)]
    pub fn render_to_string_with_width(&self, width: usize) -> String {
        self.build(width)
    }

    fn build(&self, width: usize) -> String {
        let w = width.max(30);
        let inner = w - 2; // inside the box borders
        let mut out = String::new();

        let border = style::color("border");
        let _surface = style::color("surface");
        let accent = style::color("accent");
        let text_sec = style::color("text-secondary");
        let subtle = style::color("border-subtle");

        // Top border
        out.push_str(&format!(
            "{}{}{}\n",
            border.apply_to("┌"),
            border.apply_to("─".repeat(inner)),
            border.apply_to("┐"),
        ));

        // Header line
        let title_plain = console::strip_ansi_codes(&self.title);
        let mut header = format!(" {}", accent.apply_to(&self.title));
        if let Some(sub) = &self.subtitle {
            header.push_str(&format!(" {}", text_sec.apply_to(sub)));
        }
        let right = self.count.as_deref().unwrap_or("");
        let right_len = right.len();
        let title_len = title_plain.len() + 1 + self.subtitle.as_ref().map(|s| s.len() + 1).unwrap_or(0);
        let pad = if inner > title_len + right_len + 1 {
            inner - title_len - right_len - 1
        } else {
            1
        };
        if right.is_empty() {
            header.push_str(&" ".repeat(inner.saturating_sub(title_len + 1)));
        } else {
            header.push_str(&" ".repeat(pad));
            header.push_str(&format!("{}", text_sec.apply_to(right)));
        }
        out.push_str(&format!(
            "{}{}{}\n",
            border.apply_to("│"),
            header,
            border.apply_to("│"),
        ));

        // Header separator
        out.push_str(&format!(
            "{}{}{}\n",
            border.apply_to("├"),
            border.apply_to("─".repeat(inner)),
            border.apply_to("┤"),
        ));

        // Content
        match &self.content {
            PanelContent::None => {
                let line = format!(" {:inner$}", "", inner = inner);
                out.push_str(&format!(
                    "{}{}{}\n",
                    border.apply_to("│"),
                    &line[..inner],
                    border.apply_to("│"),
                ));
            }
            PanelContent::Kv(pairs) => {
                for (key, value) in pairs {
                    let key_display = format!("{}", text_sec.apply_to(format!("{key:>16}")));
                    let val_display = format!("  {value}");
                    let visible_len = 16 + 2 + console::strip_ansi_codes(value).len();
                    let padding = inner.saturating_sub(visible_len + 2);
                    out.push_str(&format!(
                        "{} {}{}{}{}\n",
                        border.apply_to("│"),
                        key_display,
                        val_display,
                        " ".repeat(padding),
                        border.apply_to("│"),
                    ));
                }
            }
            PanelContent::Table { headers, rows } => {
                let col_count = headers.len();
                let mut widths: Vec<usize> = headers.iter().map(|h| h.len()).collect();
                for row in rows {
                    for (i, cell) in row.iter().enumerate() {
                        if i < col_count {
                            let stripped = console::strip_ansi_codes(cell);
                            widths[i] = widths[i].max(stripped.len());
                        }
                    }
                }

                // Header row
                let header_line: String = headers
                    .iter()
                    .enumerate()
                    .map(|(i, h)| {
                        format!(
                            "{}",
                            text_sec.apply_to(format!(
                                "{:<width$}",
                                h.to_uppercase(),
                                width = widths[i]
                            ))
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("  ");
                let hdr_visible: usize = widths.iter().sum::<usize>() + (col_count.saturating_sub(1)) * 2;
                let hpad = inner.saturating_sub(hdr_visible + 2);
                out.push_str(&format!(
                    "{} {}{}{}\n",
                    border.apply_to("│"),
                    header_line,
                    " ".repeat(hpad),
                    border.apply_to("│"),
                ));

                // Row separator under header
                out.push_str(&format!(
                    "{} {}{}\n",
                    border.apply_to("│"),
                    subtle.apply_to("─".repeat(inner - 2)),
                    border.apply_to("│"),
                ));

                // Data rows
                for (ri, row) in rows.iter().enumerate() {
                    let line: String = row
                        .iter()
                        .enumerate()
                        .map(|(i, cell)| {
                            let w = widths.get(i).copied().unwrap_or(cell.len());
                            let stripped = console::strip_ansi_codes(cell);
                            let is_status = headers
                                .get(i)
                                .is_some_and(|h| h.eq_ignore_ascii_case("status"));
                            if is_status {
                                let styled = style::status_dot(&stripped);
                                let spad = w.saturating_sub(stripped.len());
                                format!("{}{}", styled, " ".repeat(spad))
                            } else {
                                let pad = w.saturating_sub(stripped.len());
                                format!("{}{}", cell, " ".repeat(pad))
                            }
                        })
                        .collect::<Vec<_>>()
                        .join("  ");
                    let vis: usize = row
                        .iter()
                        .enumerate()
                        .map(|(i, cell)| {
                            let w = widths.get(i).copied().unwrap_or(0);
                            let is_status = headers
                                .get(i)
                                .is_some_and(|h| h.eq_ignore_ascii_case("status"));
                            if is_status {
                                let dot = style::status_dot(&console::strip_ansi_codes(cell));
                                let dlen = console::strip_ansi_codes(&dot).len();
                                dlen.max(w)
                            } else {
                                w
                            }
                        })
                        .sum::<usize>()
                        + (col_count.saturating_sub(1)) * 2;
                    let rpad = inner.saturating_sub(vis + 2);
                    out.push_str(&format!(
                        "{} {}{}{}\n",
                        border.apply_to("│"),
                        line,
                        " ".repeat(rpad),
                        border.apply_to("│"),
                    ));

                    // Row separator (not after last row)
                    if ri < rows.len() - 1 {
                        out.push_str(&format!(
                            "{} {}{}\n",
                            border.apply_to("│"),
                            subtle.apply_to("─".repeat(inner - 2)),
                            border.apply_to("│"),
                        ));
                    }
                }
            }
            PanelContent::Steps(steps) => {
                for (icon, label, desc) in steps {
                    let step_line = format!(" {icon} {}  {desc}", text_sec.apply_to(label));
                    let vis_len = 1 + console::strip_ansi_codes(icon).len()
                        + 1
                        + label.len()
                        + 2
                        + desc.len();
                    let spad = inner.saturating_sub(vis_len + 1);
                    out.push_str(&format!(
                        "{}{}{}{}\n",
                        border.apply_to("│"),
                        step_line,
                        " ".repeat(spad),
                        border.apply_to("│"),
                    ));
                }
            }
        }

        // Footer
        if self.footer_left.is_some() || self.footer_right.is_some() {
            out.push_str(&format!(
                "{}{}{}\n",
                border.apply_to("├"),
                border.apply_to("─".repeat(inner)),
                border.apply_to("┤"),
            ));
            let fl = self.footer_left.as_deref().unwrap_or("");
            let fr = self.footer_right.as_deref().unwrap_or("");
            let fl_len = console::strip_ansi_codes(fl).len();
            let fr_len = console::strip_ansi_codes(fr).len();
            let fpad = inner.saturating_sub(fl_len + fr_len + 2);
            out.push_str(&format!(
                "{} {}{}{} {}\n",
                border.apply_to("│"),
                fl,
                " ".repeat(fpad),
                fr,
                border.apply_to("│"),
            ));
        }

        // Bottom border
        out.push_str(&format!(
            "{}{}{}\n",
            border.apply_to("└"),
            border.apply_to("─".repeat(inner)),
            border.apply_to("┘"),
        ));

        out
    }
}

fn terminal_width() -> usize {
    console::Term::stdout()
        .size_checked()
        .map(|(_, w)| w as usize)
        .unwrap_or(80)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn panel_renders_kv_without_panic() {
        let output = Panel::new("Test")
            .kv(&[("Key1", "Value1"), ("Key2", "Value2")])
            .render_to_string();
        assert!(output.contains("Test"));
        assert!(output.contains("Key1"));
        assert!(output.contains("Value1"));
    }

    #[test]
    fn panel_renders_table_without_panic() {
        let output = Panel::new("Pods")
            .count("3 total")
            .table(
                &["Name", "Status"],
                &[
                    vec!["web".into(), "running".into()],
                    vec!["api".into(), "failed".into()],
                ],
            )
            .render_to_string();
        assert!(output.contains("Pods"));
        assert!(output.contains("3 total"));
        assert!(output.contains("web"));
    }

    #[test]
    fn panel_renders_steps_with_footer() {
        let output = Panel::new("Deploying")
            .subtitle("web-api")
            .steps(&[
                ("✓", "Image", "nginx:latest pulled"),
                ("✓", "Pod", "web-api-a1b running"),
            ])
            .footer_left("● Deployed")
            .footer_right("2/2 pods ready")
            .render_to_string();
        assert!(output.contains("Deploying"));
        assert!(output.contains("web-api"));
        assert!(output.contains("Deployed"));
    }

    #[test]
    fn panel_respects_terminal_width() {
        let output = Panel::new("Test")
            .kv(&[("Key", "Value")])
            .render_to_string_with_width(40);
        for line in output.lines() {
            let stripped = console::strip_ansi_codes(line);
            let char_count = stripped.chars().count();
            assert!(char_count <= 42, "line too long ({char_count} chars): {stripped}");
        }
    }
}
