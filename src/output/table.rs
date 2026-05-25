use super::style;

pub fn print_table(headers: &[&str], rows: &[Vec<String>]) {
    if super::is_json_mode() {
        let objects: Vec<serde_json::Value> = rows
            .iter()
            .map(|row| {
                let mut map = serde_json::Map::new();
                for (i, cell) in row.iter().enumerate() {
                    let key = headers.get(i).unwrap_or(&"").to_lowercase();
                    map.insert(key, serde_json::Value::String(cell.clone()));
                }
                serde_json::Value::Object(map)
            })
            .collect();
        super::print_json(&objects);
        return;
    }

    if rows.is_empty() {
        println!("No resources found.");
        return;
    }

    let col_count = headers.len();
    let mut widths: Vec<usize> = headers.iter().map(|h| h.len()).collect();

    for row in rows {
        for (i, cell) in row.iter().enumerate() {
            if i < col_count {
                widths[i] = widths[i].max(cell.len());
            }
        }
    }

    let text_sec = style::color("text-secondary");
    let header_line: String = headers
        .iter()
        .enumerate()
        .map(|(i, h)| {
            format!(
                "{}",
                text_sec.apply_to(format!("{:<width$}", h.to_uppercase(), width = widths[i]))
            )
        })
        .collect::<Vec<_>>()
        .join("  ");
    println!("{header_line}");

    for row in rows {
        let line: String = row
            .iter()
            .enumerate()
            .map(|(i, cell)| {
                let w = widths.get(i).copied().unwrap_or(cell.len());
                let formatted = format!("{:<width$}", cell, width = w);
                if headers
                    .get(i)
                    .is_some_and(|h| h.eq_ignore_ascii_case("status"))
                {
                    let dot = style::status_dot(cell);
                    let stripped = console::strip_ansi_codes(&dot);
                    let pad = w.saturating_sub(stripped.len());
                    return format!("{dot}{}", " ".repeat(pad));
                }
                formatted
            })
            .collect::<Vec<_>>()
            .join("  ");
        println!("{line}");
    }
}
