use std::collections::HashMap;

/// Одна метрика из Prometheus text format.
#[derive(Debug, Clone, PartialEq)]
pub struct MetricSample {
    pub name: String,
    pub labels: HashMap<String, String>,
    pub value: f64,
    pub timestamp: Option<i64>,
}

/// Парсит Prometheus text format → Vec<MetricSample>.
/// Игнорирует # HELP / # TYPE строки и строки с ошибками (без panic).
pub fn parse(input: &str) -> Vec<MetricSample> {
    let mut samples = Vec::new();
    for line in input.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        match parse_line(line) {
            Some(s) => samples.push(s),
            None => tracing::trace!(line, "metrics: skipping unparseable line"),
        }
    }
    samples
}

/// Парсит одну строку: `name{k="v",...} value [timestamp]`
fn parse_line(line: &str) -> Option<MetricSample> {
    // Split off optional timestamp (last token if it has >1 space-separated part after value)
    // Format:  name_part value [timestamp]
    //   where name_part = name  |  name{labels}
    let (name_part, rest) = if let Some(brace) = line.find('{') {
        // Has labels
        let close = line.find('}')?;
        let name = line[..brace].to_string();
        let labels = parse_labels(&line[brace + 1..close])?;
        let after = line[close + 1..].trim(); // " value [ts]"
        return parse_value_ts(name, labels, after);
    } else {
        // No labels
        let mut parts = line.splitn(2, ' ');
        let name = parts.next()?.to_string();
        let after = parts.next()?.trim();
        (name, after)
    };

    parse_value_ts(name_part, HashMap::new(), rest)
}

fn parse_value_ts(
    name: String,
    labels: HashMap<String, String>,
    rest: &str,
) -> Option<MetricSample> {
    let mut tokens = rest.split_whitespace();
    let value_str = tokens.next()?;
    let value = value_str.parse::<f64>().ok()?;
    let timestamp = tokens.next().and_then(|t| t.parse::<i64>().ok());
    Some(MetricSample {
        name,
        labels,
        value,
        timestamp,
    })
}

/// Parses `key="val",key2="val2"` → HashMap
fn parse_labels(s: &str) -> Option<HashMap<String, String>> {
    let mut map = HashMap::new();
    for pair in s.split(',') {
        let pair = pair.trim();
        if pair.is_empty() {
            continue;
        }
        let mut kv = pair.splitn(2, '=');
        let key = kv.next()?.trim().to_string();
        let val = kv.next()?.trim().trim_matches('"').to_string();
        map.insert(key, val);
    }
    Some(map)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_simple_metric() {
        let samples = parse("zt_packet_rx_bytes 12345");
        assert_eq!(samples.len(), 1);
        assert_eq!(samples[0].name, "zt_packet_rx_bytes");
        assert_eq!(samples[0].value, 12345.0);
        assert!(samples[0].labels.is_empty());
    }

    #[test]
    fn parses_metric_with_labels() {
        let samples = parse(r#"zt_peer_latency{node_id="deadbeef01",status="online"} 42.5"#);
        assert_eq!(samples.len(), 1);
        assert_eq!(samples[0].name, "zt_peer_latency");
        assert_eq!(samples[0].value, 42.5);
        assert_eq!(
            samples[0].labels.get("node_id").map(|s| s.as_str()),
            Some("deadbeef01")
        );
        assert_eq!(
            samples[0].labels.get("status").map(|s| s.as_str()),
            Some("online")
        );
    }

    #[test]
    fn skips_comments_and_empty() {
        let input = "# HELP zt_foo help text\n# TYPE zt_foo gauge\nzt_foo 1.0\n\n";
        let samples = parse(input);
        assert_eq!(samples.len(), 1);
        assert_eq!(samples[0].name, "zt_foo");
    }

    #[test]
    fn parses_with_timestamp() {
        let samples = parse("zt_foo 9.0 1609459200000");
        assert_eq!(samples[0].timestamp, Some(1609459200000));
    }

    #[test]
    fn skips_unparseable_without_panic() {
        let samples = parse("not_a_valid_line_with_no_value_xxx");
        assert_eq!(samples.len(), 0);
    }

    #[test]
    fn parses_multiple_metrics() {
        let input = "zt_a 1\nzt_b 2\nzt_c 3\n";
        assert_eq!(parse(input).len(), 3);
    }
}
