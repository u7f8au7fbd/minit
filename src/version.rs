use std::cmp::Ordering;

pub fn has_alpha(value: &str) -> bool {
    value.chars().any(|ch| ch.is_ascii_alphabetic())
}

pub fn sort_desc(versions: &mut [String]) {
    versions.sort_by(|left, right| compare_versions(right, left));
}

fn compare_versions(left: &str, right: &str) -> Ordering {
    let left_parts = numeric_parts(left);
    let right_parts = numeric_parts(right);
    let max_len = left_parts.len().max(right_parts.len());

    for index in 0..max_len {
        let left = left_parts.get(index).copied().unwrap_or(0);
        let right = right_parts.get(index).copied().unwrap_or(0);
        match left.cmp(&right) {
            Ordering::Equal => {}
            order => return order,
        }
    }

    match has_alpha(left).cmp(&has_alpha(right)) {
        Ordering::Equal => left.cmp(right),
        order => order.reverse(),
    }
}

fn numeric_parts(value: &str) -> Vec<u32> {
    let mut parts = Vec::new();
    let mut current = String::new();

    for ch in value.chars() {
        if ch.is_ascii_digit() {
            current.push(ch);
        } else if !current.is_empty() {
            if let Ok(part) = current.parse() {
                parts.push(part);
            }
            current.clear();
        }
    }

    if !current.is_empty() {
        if let Ok(part) = current.parse() {
            parts.push(part);
        }
    }

    parts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sorts_numeric_segments_descending() {
        let mut versions = vec![
            "1.21.9".to_string(),
            "1.21.11".to_string(),
            "1.21.1".to_string(),
            "1.20.6".to_string(),
        ];

        sort_desc(&mut versions);

        assert_eq!(versions, vec!["1.21.11", "1.21.9", "1.21.1", "1.20.6"]);
    }

    #[test]
    fn detects_alpha_versions() {
        assert!(!has_alpha("1.21.11"));
        assert!(has_alpha("1.21.11-rc1"));
        assert!(has_alpha("21.4.111-beta"));
    }
}
