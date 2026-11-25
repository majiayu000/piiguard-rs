use super::PatternRecognizer;

const PATTERN: &str = r"[A-Za-z0-9._%+\-]+@[A-Za-z0-9\-]+(?:\.[A-Za-z0-9\-]+)*\.[A-Za-z]{2,24}";

fn validate(s: &str) -> Option<f32> {
    let (local, domain) = s.split_once('@')?;
    let bad_local = local.starts_with('.') || local.ends_with('.') || local.contains("..");
    let bad_domain = domain
        .split('.')
        .any(|l| l.starts_with('-') || l.ends_with('-'));
    (!bad_local && !bad_domain && local.len() <= 64).then_some(0.9)
}

pub fn recognizer() -> Result<PatternRecognizer, regex::Error> {
    Ok(PatternRecognizer::new("EMAIL", PATTERN, 0.9)?.validator(validate))
}
