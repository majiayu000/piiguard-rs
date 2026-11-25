use super::PatternRecognizer;

/// URL 查询参数里的凭据值：按参数名识别，只标出 `=` 之后的值。
/// 这里不识别密钥本身的格式（`sk-`、`AKIA` 等前缀不在本期范围）。
const PATTERN: &str = concat!(
    r"[?&;](?i:access_token|refresh_token|id_token|auth_token|token|auth|",
    r"session_id|sessionid|session|sid|signature|sig|password|passwd|pwd|secret|",
    r"api_key|apikey|client_secret)=([^&#\s]+)",
);

/// 过短的值（`token=1`）多半是开关或占位，不报。
fn validate(s: &str) -> Option<f32> {
    (s.chars().count() >= 6).then_some(0.75)
}

pub fn recognizer() -> Result<PatternRecognizer, regex::Error> {
    Ok(PatternRecognizer::new("URL_TOKEN", PATTERN, 0.75)?
        .validator(validate)
        .unbounded())
}
