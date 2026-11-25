//! JSON 配置：选择内置识别器、阈值、允许/拒绝列表、自定义正则识别器。需要 `serde` feature。

use serde::Deserialize;

use crate::analyzer::{Analyzer, AnalyzerBuilder};
use crate::error::Error;
use crate::recognizers::PatternRecognizer;

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// 启用的内置实体；省略表示全部启用，空数组表示不启用内置识别器。
    pub entities: Option<Vec<String>>,
    pub threshold: Option<f32>,
    #[serde(default)]
    pub allow: Vec<String>,
    #[serde(default)]
    pub deny: Vec<DenyRule>,
    #[serde(default)]
    pub custom: Vec<CustomRule>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DenyRule {
    pub entity: String,
    pub values: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CustomRule {
    pub entity: String,
    pub pattern: String,
    pub score: f32,
    #[serde(default)]
    pub context: Vec<String>,
    #[serde(default)]
    pub context_boost: f32,
}

impl Config {
    pub fn from_json(s: &str) -> Result<Self, Error> {
        serde_json::from_str(s).map_err(Error::Config)
    }

    pub fn into_builder(self) -> Result<AnalyzerBuilder, Error> {
        let mut b = match &self.entities {
            Some(list) => Analyzer::builder().with_builtin(list),
            None => Analyzer::builder().with_all_builtin(),
        };
        if let Some(t) = self.threshold {
            b = b.threshold(t);
        }
        b = b.allow(&self.allow);
        for d in &self.deny {
            b = b.deny(&d.entity, &d.values);
        }
        for c in self.custom {
            let r = PatternRecognizer::new(c.entity, &c.pattern, c.score)?
                .context(&c.context, c.context_boost);
            b = b.with_recognizer(r);
        }
        Ok(b)
    }
}
