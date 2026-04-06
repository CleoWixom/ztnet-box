use std::collections::HashMap;

/// Простой парсер Prometheus text format.
pub fn parse(input: &str) -> HashMap<String, f64> {
    let mut map = HashMap::new();
    for line in input.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        // Format: metric_name{labels} value [timestamp]
        // or:     metric_name value [timestamp]
        let mut parts = line.splitn(2, ' ');
        let name_part = parts
            .next()
            .unwrap_or("")
            .split('{')
            .next()
            .unwrap_or("")
            .to_string();
        if let Some(val_part) = parts.next() {
            let value_str = val_part.split_whitespace().next().unwrap_or("");
            if let Ok(v) = value_str.parse::<f64>() {
                map.insert(name_part, v);
            }
        }
    }
    map
}
