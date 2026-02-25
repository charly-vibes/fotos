/// PII (Personally Identifiable Information) detection.
///
/// Runs regex pattern matching on OCR-extracted text regions to identify
/// sensitive information and return bounding boxes for each match.
use anyhow::Result;
use regex::Regex;

pub struct PiiMatch {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
    pub pii_type: String,
    pub text: String,
}

/// Detect PII in OCR-extracted word regions.
///
/// Concatenates all region text into one string (tracking each word's byte
/// offsets), runs each regex against the full text, then unions the bounding
/// boxes of every word that overlaps each match.
pub fn detect_pii(ocr_regions: &[super::ocr::OcrRegion]) -> Result<Vec<PiiMatch>> {
    if ocr_regions.is_empty() {
        return Ok(vec![]);
    }

    // Build a single string from all word regions, tracking each word's
    // byte offsets so we can map regex matches back to pixel coordinates.
    struct WordSpan {
        start: usize,
        end: usize,
        region_idx: usize,
    }

    let mut full_text = String::new();
    let mut word_spans: Vec<WordSpan> = Vec::new();

    for (i, region) in ocr_regions.iter().enumerate() {
        let start = full_text.len();
        full_text.push_str(&region.text);
        let end = full_text.len();
        word_spans.push(WordSpan { start, end, region_idx: i });
        full_text.push(' ');
    }

    // (pii_type, regex_pattern)
    let patterns: &[(&str, &str)] = &[
        ("email",       r"[a-zA-Z0-9._%+\-]+@[a-zA-Z0-9.\-]+\.[a-zA-Z]{2,}"),
        ("phone",       r"(?:\+1[\s\-]?)?\(?\d{3}\)?[\s\-]?\d{3}[\s\-]?\d{4}"),
        ("ssn",         r"\b\d{3}-\d{2}-\d{4}\b"),
        ("credit_card", r"\b(?:\d{4}[\s\-]?){3}\d{4}\b"),
        ("ip_v4",       r"\b(?:25[0-5]|2\d{2}|1\d{2}|[1-9]\d|\d)(?:\.(?:25[0-5]|2\d{2}|1\d{2}|[1-9]\d|\d)){3}\b"),
        ("ip_v6",       r"\b(?:[0-9a-fA-F]{1,4}:){2,7}[0-9a-fA-F]{0,4}\b"),
        ("api_key",     r"\b(?:sk|pk)[-_][a-zA-Z0-9]{16,}|ghp_[a-zA-Z0-9]{36}|AKIA[A-Z0-9]{16}\b"),
        ("url",         r"https?://[^\s]+"),
    ];

    let mut matches = Vec::new();

    for &(pii_type, pattern) in patterns {
        let re = Regex::new(pattern)?;
        for mat in re.find_iter(&full_text) {
            let match_start = mat.start();
            let match_end = mat.end();
            let matched_text = mat.as_str().trim().to_string();

            // Credit cards: validate with Luhn algorithm to cut false positives.
            if pii_type == "credit_card" {
                let digits: String = matched_text
                    .chars()
                    .filter(|c| c.is_ascii_digit())
                    .collect();
                if !luhn_valid(&digits) {
                    continue;
                }
            }

            // Find every word span that overlaps this match.
            let overlapping: Vec<&WordSpan> = word_spans
                .iter()
                .filter(|span| span.start < match_end && span.end > match_start)
                .collect();

            if overlapping.is_empty() {
                continue;
            }

            // Union all overlapping word bounding boxes.
            let bbox = overlapping
                .iter()
                .map(|span| &ocr_regions[span.region_idx])
                .fold(None::<(u32, u32, u32, u32)>, |acc, r| {
                    let x2 = r.x + r.w;
                    let y2 = r.y + r.h;
                    Some(match acc {
                        None => (r.x, r.y, x2, y2),
                        Some((ax, ay, ax2, ay2)) => {
                            (ax.min(r.x), ay.min(r.y), ax2.max(x2), ay2.max(y2))
                        }
                    })
                });

            if let Some((x, y, x2, y2)) = bbox {
                matches.push(PiiMatch {
                    x,
                    y,
                    w: x2 - x,
                    h: y2 - y,
                    pii_type: pii_type.to_string(),
                    text: matched_text,
                });
            }
        }
    }

    Ok(matches)
}

/// Luhn algorithm check for credit card numbers.
fn luhn_valid(digits: &str) -> bool {
    if digits.len() < 13 || digits.len() > 19 {
        return false;
    }
    let sum: u32 = digits
        .chars()
        .rev()
        .enumerate()
        .map(|(i, c)| {
            let d = c.to_digit(10).unwrap_or(0);
            if i % 2 == 1 {
                let doubled = d * 2;
                if doubled > 9 { doubled - 9 } else { doubled }
            } else {
                d
            }
        })
        .sum();
    sum.is_multiple_of(10)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::ocr::OcrRegion;

    fn region(text: &str, x: u32, y: u32, w: u32, h: u32) -> OcrRegion {
        OcrRegion { text: text.into(), x, y, w, h, confidence: 1.0 }
    }

    #[test]
    fn empty_regions_returns_empty() {
        let result = detect_pii(&[]).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn detects_email() {
        let regions = [region("user@example.com", 0, 0, 100, 20)];
        let matches = detect_pii(&regions).unwrap();
        assert!(matches.iter().any(|m| m.pii_type == "email"));
    }

    #[test]
    fn detects_ssn() {
        let regions = [region("123-45-6789", 10, 20, 80, 16)];
        let matches = detect_pii(&regions).unwrap();
        assert!(matches.iter().any(|m| m.pii_type == "ssn"));
    }

    #[test]
    fn detects_ipv4() {
        let regions = [region("192.168.1.1", 0, 0, 60, 16)];
        let matches = detect_pii(&regions).unwrap();
        assert!(matches.iter().any(|m| m.pii_type == "ip_v4"));
    }

    #[test]
    fn detects_url() {
        let regions = [region("https://example.com/path", 0, 0, 150, 16)];
        let matches = detect_pii(&regions).unwrap();
        assert!(matches.iter().any(|m| m.pii_type == "url"));
    }

    #[test]
    fn detects_api_key_sk() {
        let regions = [region("sk-abcdefghijklmnopqrstuvwxyz123456", 0, 0, 200, 16)];
        let matches = detect_pii(&regions).unwrap();
        assert!(matches.iter().any(|m| m.pii_type == "api_key"), "{:?}", matches.iter().map(|m| &m.pii_type).collect::<Vec<_>>());
    }

    #[test]
    fn detects_akia_key() {
        let regions = [region("AKIAIOSFODNN7EXAMPLE", 0, 0, 200, 16)];
        let matches = detect_pii(&regions).unwrap();
        assert!(matches.iter().any(|m| m.pii_type == "api_key"));
    }

    #[test]
    fn credit_card_luhn_valid_detected() {
        // Visa test number: 4111111111111111 (Luhn valid)
        let regions = [region("4111111111111111", 0, 0, 120, 16)];
        let matches = detect_pii(&regions).unwrap();
        assert!(matches.iter().any(|m| m.pii_type == "credit_card"));
    }

    #[test]
    fn credit_card_luhn_invalid_skipped() {
        // Invalid card number
        let regions = [region("1234567890123456", 0, 0, 120, 16)];
        let matches = detect_pii(&regions).unwrap();
        assert!(!matches.iter().any(|m| m.pii_type == "credit_card"));
    }

    #[test]
    fn multi_word_match_unions_bboxes() {
        // Email split across two words due to OCR tokenization
        // (single word here, verifying bbox is correct)
        let regions = [region("admin@corp.io", 5, 10, 90, 18)];
        let matches = detect_pii(&regions).unwrap();
        let m = matches.iter().find(|m| m.pii_type == "email").unwrap();
        assert_eq!(m.x, 5);
        assert_eq!(m.y, 10);
        assert_eq!(m.w, 90);
        assert_eq!(m.h, 18);
    }

    #[test]
    fn luhn_valid_visa_test_card() {
        assert!(luhn_valid("4111111111111111"));
    }

    #[test]
    fn luhn_invalid_random_digits() {
        assert!(!luhn_valid("1234567890123456"));
    }

    #[test]
    fn luhn_rejects_wrong_length() {
        assert!(!luhn_valid("123"));
        assert!(!luhn_valid("12345678901234567890"));
    }
}
