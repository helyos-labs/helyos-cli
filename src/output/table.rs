use console::Style;

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

    let bold = Style::new().bold();
    let header_line: String = headers
        .iter()
        .enumerate()
        .map(|(i, h)| {
            format!(
                "{}",
                bold.apply_to(format!("{:<width$}", h.to_uppercase(), width = widths[i]))
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
                    let s = match cell.to_lowercase().as_str() {
                        "running" => Style::new().green(),
                        "degraded" | "restarting" => Style::new().yellow(),
                        "failed" | "crashloopbackoff" => Style::new().red(),
                        "stopped" | "stopping" => Style::new().dim(),
                        "pending" | "creating" => Style::new().cyan(),
                        _ => Style::new(),
                    };
                    return format!("{}", s.apply_to(formatted));
                }
                formatted
            })
            .collect::<Vec<_>>()
            .join("  ");
        println!("{line}");
    }
}
