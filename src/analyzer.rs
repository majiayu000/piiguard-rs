//! `Analyzer`：逐个调用识别器，再按阈值过滤、去重叠。
//!
//! 不用 `RegexSet` 预筛：实测这批以数字计数重复为主的模式合成一个 `RegexSet` 后，
//! 仅 `matches()` 就只有 16–24 MB/s，而各自独立扫描合计约 85 MB/s（见 docs/benchmark.md）。

use std::collections::HashSet;

use crate::error::Error;
use crate::recognizers::{BUILTIN_ENTITIES, Finding, PatternRecognizer, Recognizer, builtin};

/// 默认分数阈值：低于它的命中不报。
pub const DEFAULT_THRESHOLD: f32 = 0.5;

pub struct Analyzer {
    recognizers: Vec<Box<dyn Recognizer>>,
    threshold: f32,
    allow: HashSet<String>,
}

pub struct AnalyzerBuilder {
    entities: Vec<String>,
    custom: Vec<Box<dyn Recognizer>>,
    deny: Vec<(String, Vec<String>)>,
    allow: HashSet<String>,
    threshold: f32,
}

impl Analyzer {
    pub fn builder() -> AnalyzerBuilder {
        AnalyzerBuilder {
            entities: Vec::new(),
            custom: Vec::new(),
            deny: Vec::new(),
            allow: HashSet::new(),
            threshold: DEFAULT_THRESHOLD,
        }
    }

    /// 返回去重叠、按位置排序、分数 ≥ 阈值的命中。
    pub fn analyze(&self, text: &str) -> Vec<Finding> {
        let mut raw = Vec::new();
        for r in &self.recognizers {
            r.find(text, &mut raw);
        }
        raw.retain(|f| f.score >= self.threshold && !self.allow.contains(&text[f.start..f.end]));
        let mut out = remove_overlaps(raw);
        fill_char_offsets(text, &mut out);
        out
    }
}

impl AnalyzerBuilder {
    pub fn with_builtin<S: AsRef<str>>(mut self, entities: &[S]) -> Self {
        self.entities
            .extend(entities.iter().map(|e| e.as_ref().to_string()));
        self
    }

    pub fn with_all_builtin(self) -> Self {
        self.with_builtin(BUILTIN_ENTITIES)
    }

    pub fn with_recognizer(mut self, r: impl Recognizer + 'static) -> Self {
        self.custom.push(Box::new(r));
        self
    }

    /// 拒绝列表：这些字面值一律以 `entity` 报出，分数 1.0。
    pub fn deny<S: AsRef<str>>(mut self, entity: &str, values: &[S]) -> Self {
        let values = values.iter().map(|v| v.as_ref().to_string()).collect();
        self.deny.push((entity.to_string(), values));
        self
    }

    /// 允许列表：命中原文与这些值完全相同时不报。
    pub fn allow<S: AsRef<str>>(mut self, values: &[S]) -> Self {
        self.allow
            .extend(values.iter().map(|v| v.as_ref().to_string()));
        self
    }

    pub fn threshold(mut self, t: f32) -> Self {
        self.threshold = t;
        self
    }

    pub fn build(self) -> Result<Analyzer, Error> {
        let mut recognizers: Vec<Box<dyn Recognizer>> = Vec::new();
        for e in &self.entities {
            let r = builtin(e)?.ok_or_else(|| Error::UnknownEntity(e.clone()))?;
            recognizers.push(Box::new(r));
        }
        for (entity, values) in self.deny.iter().filter(|(_, v)| !v.is_empty()) {
            let alt: Vec<String> = values.iter().map(|v| regex::escape(v)).collect();
            recognizers.push(Box::new(PatternRecognizer::new(
                entity.as_str(),
                &alt.join("|"),
                1.0,
            )?));
        }
        recognizers.extend(self.custom);

        Ok(Analyzer {
            recognizers,
            threshold: self.threshold,
            allow: self.allow,
        })
    }
}

/// 重叠时保留分数高的；同分保留更长的；再同则保留靠前的。
fn remove_overlaps(mut raw: Vec<Finding>) -> Vec<Finding> {
    raw.sort_by(|a, b| {
        b.score
            .total_cmp(&a.score)
            .then((b.end - b.start).cmp(&(a.end - a.start)))
            .then(a.start.cmp(&b.start))
    });
    let mut kept: Vec<Finding> = Vec::with_capacity(raw.len());
    for f in raw {
        let p = kept.partition_point(|k| k.start < f.start);
        let left_clear = p == 0 || kept[p - 1].end <= f.start;
        let right_clear = p == kept.len() || kept[p].start >= f.end;
        if left_clear && right_clear {
            kept.insert(p, f);
        }
    }
    kept
}

/// `findings` 已按 `start` 排序且互不重叠，一次线性扫描回填字符偏移。
fn fill_char_offsets(text: &str, findings: &mut [Finding]) {
    let (mut byte, mut chars) = (0usize, 0usize);
    for f in findings {
        chars += text[byte..f.start].chars().count();
        f.char_start = chars;
        chars += text[f.start..f.end].chars().count();
        f.char_end = chars;
        byte = f.end;
    }
}
