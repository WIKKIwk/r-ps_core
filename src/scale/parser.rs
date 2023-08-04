use regex::Regex;
use std::sync::LazyLock;

static WEIGHT_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)([-+N]?)\s*(\d+(?:[.,]\d+)?)\s*(kg|g|lb|lbs|oz)?\s*([-+]?)")
        .expect("weight regex is valid")
});

static STABLE_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)\bST\b|\bSTABLE\b").expect("stable regex is valid"));

static UNSTABLE_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)\bUS\b|\bUNSTABLE\b").expect("unstable regex is valid"));

#[derive(Debug, Clone, PartialEq)]
pub struct ParsedWeight {
    pub weight: f64,
    pub unit: String,
    pub stable: Option<bool>,
}

#[derive(Debug, Clone)]
struct WeightCandidate {
    weight: f64,
    unit: String,
    index: usize,
    has_unit: bool,
    has_explicit_sign: bool,
    has_negative_sign: bool,
    has_positive_sign: bool,
}

pub fn parse_weight(raw: &str, default_unit: &str) -> Option<ParsedWeight> {
    let normalized = normalize_minus(raw);
    let mut candidates = Vec::new();

    for captures in WEIGHT_REGEX.captures_iter(&normalized) {
        let whole_match = match captures.get(0) {
            Some(value) => value,
            None => continue,
        };
        let prefix = capture_text(&captures, 1);
        let number_part = capture_text(&captures, 2);
        let unit_part = capture_text(&captures, 3).trim().to_ascii_lowercase();
        let suffix = capture_text(&captures, 4);

        let sign = resolve_sign(prefix, suffix);
        let mut number = number_part.trim().replace(',', ".");
        if let Some(sign_value) = sign.sign {
            number.insert(0, sign_value);
        }

        let weight = match number.parse::<f64>() {
            Ok(value) => value,
            Err(_) => continue,
        };
        if !(-1_000_000.0..=1_000_000.0).contains(&weight) {
            continue;
        }

        let unit = if unit_part.is_empty() {
            default_unit.trim().to_ascii_lowercase()
        } else {
            unit_part.clone()
        };

        candidates.push(WeightCandidate {
            weight,
            unit,
            index: whole_match.start(),
            has_unit: !unit_part.is_empty(),
            has_explicit_sign: sign.has_sign,
            has_negative_sign: sign.negative,
            has_positive_sign: sign.positive,
        });
    }

    let best = best_candidate(candidates)?;
    let stable = if UNSTABLE_REGEX.is_match(&normalized) {
        Some(false)
    } else if STABLE_REGEX.is_match(&normalized) {
        Some(true)
    } else {
        None
    };

    Some(ParsedWeight {
        weight: best.weight,
        unit: best.unit,
        stable,
    })
}

pub fn stable_text(stable: Option<bool>) -> &'static str {
    match stable {
        Some(true) => "stable",
        Some(false) => "unstable",
        None => "unknown",
    }
}

fn capture_text<'a>(captures: &'a regex::Captures<'a>, index: usize) -> &'a str {
    captures
        .get(index)
        .map(|value| value.as_str())
        .unwrap_or("")
}

fn best_candidate(candidates: Vec<WeightCandidate>) -> Option<WeightCandidate> {
    candidates.into_iter().max_by(|left, right| {
        let left_score = score_candidate(left);
        let right_score = score_candidate(right);
        left_score
            .cmp(&right_score)
            .then_with(|| left.index.cmp(&right.index))
    })
}

fn score_candidate(candidate: &WeightCandidate) -> i32 {
    let mut score = 0;
    if candidate.has_unit {
        score += 80;
    }
    if candidate.has_explicit_sign {
        score += 40;
    }
    if candidate.has_negative_sign {
        score += 120;
    }
    if candidate.has_positive_sign {
        score += 10;
    }
    score
}

#[derive(Debug, Clone, Copy)]
struct ResolvedSign {
    sign: Option<char>,
    has_sign: bool,
    negative: bool,
    positive: bool,
}

fn resolve_sign(prefix: &str, suffix: &str) -> ResolvedSign {
    let prefix = prefix.trim().to_ascii_uppercase();
    let suffix = suffix.trim();

    match (prefix.as_str(), suffix) {
        ("-", _) | (_, "-") => negative_sign(),
        ("N", _) => negative_sign(),
        ("+", _) | (_, "+") => positive_sign(),
        _ => ResolvedSign {
            sign: None,
            has_sign: false,
            negative: false,
            positive: false,
        },
    }
}

fn negative_sign() -> ResolvedSign {
    ResolvedSign {
        sign: Some('-'),
        has_sign: true,
        negative: true,
        positive: false,
    }
}

fn positive_sign() -> ResolvedSign {
    ResolvedSign {
        sign: Some('+'),
        has_sign: true,
        negative: false,
        positive: true,
    }
}

fn normalize_minus(raw: &str) -> String {
    raw.replace(['\u{2212}', '\u{2013}', '\u{2014}'], "-")
}

#[cfg(test)]
mod tests {
    use super::{parse_weight, stable_text};

    #[test]
    fn parses_negative_formats_like_go() {
        let cases = [
            ("ST,-13kg", -13.0),
            ("ST, - 13 kg", -13.0),
            ("ST, 13 kg-", -13.0),
            ("ST, \u{2212}13.5kg", -13.5),
            ("ST, N13.25kg", -13.25),
        ];

        for (raw, expected) in cases {
            let parsed = parse_weight(raw, "kg").expect("parse weight");
            assert_eq!(parsed.weight, expected, "raw={raw}");
            assert_eq!(parsed.unit, "kg");
        }
    }

    #[test]
    fn prefers_negative_when_frame_contains_both_signs_like_go() {
        let parsed = parse_weight("x=13kg net=-13kg ST", "kg").expect("parse weight");
        assert_eq!(parsed.weight, -13.0);
        assert_eq!(parsed.unit, "kg");
        assert_eq!(parsed.stable, Some(true));
    }

    #[test]
    fn parses_positive_format_like_go() {
        let parsed = parse_weight("ST, +13.40kg", "kg").expect("parse weight");
        assert_eq!(parsed.weight, 13.4);
        assert_eq!(parsed.unit, "kg");
        assert_eq!(parsed.stable, Some(true));
    }

    #[test]
    fn unstable_marker_wins_over_stable_marker_like_go() {
        let parsed = parse_weight("ST US 10kg", "kg").expect("parse weight");
        assert_eq!(parsed.stable, Some(false));
        assert_eq!(stable_text(parsed.stable), "unstable");
    }

    #[test]
    fn chooses_later_candidate_on_equal_score_like_go() {
        let parsed = parse_weight("first 12 second 13", "kg").expect("parse weight");
        assert_eq!(parsed.weight, 13.0);
    }

    #[test]
    fn rejects_out_of_range_values_like_go() {
        assert!(parse_weight("1000001kg", "kg").is_none());
        assert!(parse_weight("-1000001kg", "kg").is_none());
    }
}
