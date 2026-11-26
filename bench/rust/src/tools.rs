//! 各工具的适配层：统一成 `detect(text) -> Vec<(字节起, 字节止, 原始标签)>`。
//! 每个工具都用其默认/推荐配置；偏离默认的地方在注释里写明。

use std::path::Path;

pub const NAMES: &[&str] = &["piiguard", "presidio-rs", "pii-vault", "pii-veil", "cloakpipe", "llm-pii-redact"];

pub enum Tool {
    Piiguard(piiguard::Analyzer),
    PresidioRs(presidio_analyzer::AnalyzerEngine),
    PiiVault(pii_vault::Analyzer),
    PiiVeil(pii_veil_core::DetectorPipeline),
    Cloakpipe(cloakpipe_core::detector::Detector),
    LlmPiiRedact(llm_pii_redact::Redactor),
}

type Span = (usize, usize, String);

impl Tool {
    pub fn load(name: &str) -> Result<Self, String> {
        Ok(match name {
            "piiguard" => Self::Piiguard(
                piiguard::Analyzer::builder().with_all_builtin().build().map_err(|e| e.to_string())?,
            ),
            // 默认引擎：全部英文预置识别器 + 国家识别器 + SimpleNlpEngine，阈值 0。
            "presidio-rs" => Self::PresidioRs(presidio_analyzer::AnalyzerEngine::new()),
            // crate 不带识别规则，规则来自同版本 git tag v0.2.0 的 spec/recognizers（由 run.sh 拉取）。
            "pii-vault" => {
                let dir = std::env::var("PII_VAULT_SPEC").map_err(|_| "PII_VAULT_SPEC not set")?;
                let recs = pii_vault::load_recognizers_from_dir(Path::new(&dir));
                if recs.is_empty() {
                    return Err(format!("no recognizers in {dir}"));
                }
                Self::PiiVault(pii_vault::Analyzer::new(recs))
            }
            "pii-veil" => Self::PiiVeil(pii_veil_core::DetectorPipeline::new()),
            // 默认配置关闭了电话与 IP；这里打开它们（工具本身支持），其余保持默认。
            "cloakpipe" => {
                let cfg: cloakpipe_core::config::DetectionConfig =
                    serde_json::from_str(r#"{"phone_numbers": true, "ip_addresses": true}"#)
                        .map_err(|e| e.to_string())?;
                Self::Cloakpipe(
                    cloakpipe_core::detector::Detector::from_config(&cfg).map_err(|e| e.to_string())?,
                )
            }
            "llm-pii-redact" => Self::LlmPiiRedact(llm_pii_redact::Redactor::default()),
            other => return Err(format!("unknown tool {other}")),
        })
    }

    pub fn detect(&self, text: &str) -> Vec<Span> {
        match self {
            Self::Piiguard(a) => a.analyze(text).into_iter().map(|f| (f.start, f.end, f.entity)).collect(),
            Self::PresidioRs(e) => e
                .analyze(text, "en", None, None)
                .into_iter()
                .map(|r| (r.start, r.end, r.entity_type))
                .collect(),
            Self::PiiVault(a) => a
                .analyze(text, &[], 0.0)
                .entities
                .into_iter()
                .map(|r| (r.start, r.end, r.entity_type.to_string()))
                .collect(),
            Self::PiiVeil(p) => p
                .detect(text)
                .into_iter()
                .map(|s| (s.start, s.end, s.kind.label().to_string()))
                .collect(),
            Self::Cloakpipe(d) => d
                .detect(text)
                .unwrap_or_default()
                .into_iter()
                .map(|e| (e.start, e.end, format!("{:?}", e.category)))
                .collect(),
            Self::LlmPiiRedact(r) => r.detect(text).into_iter().map(|d| (d.start, d.end, d.kind)).collect(),
        }
    }
}
