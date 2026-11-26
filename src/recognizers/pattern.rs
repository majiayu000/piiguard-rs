use regex::Regex;

use super::context::{preceded_by, window_has};
use super::{Finding, Recognizer};

/// 校验函数：输入命中的原文，合法时返回基础分，不合法返回 `None`。
pub type Validator = fn(&str) -> Option<f32>;

/// 正则 + 校验函数 + 上下文词的通用识别器。内置识别器、拒绝列表、自定义识别器都基于它。
///
/// - 正则若有捕获组，命中区间取第 1 组；否则取整个匹配。
/// - 默认要求命中两侧不能紧挨 ASCII 字母数字（不用 `\b`，因为 Unicode `\b` 会把
///   "身份证110…" 中的汉字和数字视为同一个词）。
pub struct PatternRecognizer {
    entity: String,
    regex: Regex,
    score: f32,
    validator: Option<Validator>,
    context: Vec<String>,
    context_boost: f32,
    negative: Vec<String>,
    negative_penalty: f32,
    bounded: bool,
}

impl PatternRecognizer {
    pub fn new(entity: impl Into<String>, pattern: &str, score: f32) -> Result<Self, regex::Error> {
        Ok(Self {
            entity: entity.into(),
            regex: Regex::new(pattern)?,
            score,
            validator: None,
            context: Vec::new(),
            context_boost: 0.0,
            negative: Vec::new(),
            negative_penalty: 0.0,
            bounded: true,
        })
    }

    /// 设置校验函数；设置后基础分以校验函数返回值为准。
    pub fn validator(mut self, f: Validator) -> Self {
        self.validator = Some(f);
        self
    }

    /// 命中前的窗口里出现任一上下文词（不区分大小写）时，分数加 `boost`，封顶 1.0。
    pub fn context<S: AsRef<str>>(mut self, words: &[S], boost: f32) -> Self {
        self.context = words.iter().map(|w| w.as_ref().to_lowercase()).collect();
        self.context_boost = boost;
        self
    }

    /// 命中前紧挨着（跳过空格、`=`、`:` 等分隔符）任一负向词时，分数减 `penalty`。
    /// 用于 `ver=1.2.3.4`、`order=4111…` 这类"长得像 PII 但字段名说明不是"的场景。
    pub fn negative_context<S: AsRef<str>>(mut self, words: &[S], penalty: f32) -> Self {
        self.negative = words.iter().map(|w| w.as_ref().to_lowercase()).collect();
        self.negative_penalty = penalty;
        self
    }

    /// 关闭两侧边界检查（如 URL 参数值本身就紧跟在 `=` 之后）。
    pub fn unbounded(mut self) -> Self {
        self.bounded = false;
        self
    }

    fn boundary_ok(&self, text: &str, start: usize, end: usize) -> bool {
        if !self.bounded {
            return true;
        }
        let before = text[..start].chars().next_back();
        let after = text[end..].chars().next();
        !before.is_some_and(|c| c.is_ascii_alphanumeric())
            && !after.is_some_and(|c| c.is_ascii_alphanumeric())
    }

    fn push(&self, text: &str, start: usize, end: usize, out: &mut Vec<Finding>) {
        if start == end || !self.boundary_ok(text, start, end) {
            return;
        }
        let base = match self.validator {
            Some(f) => match f(&text[start..end]) {
                Some(s) => s,
                None => return,
            },
            None => self.score,
        };
        let mut score = base;
        if window_has(text, start, &self.context) {
            score = (score + self.context_boost).min(1.0);
        }
        if preceded_by(text, start, &self.negative) {
            score -= self.negative_penalty;
        }
        out.push(Finding::new(self.entity.as_str(), start, end, score));
    }
}

impl Recognizer for PatternRecognizer {
    fn entity(&self) -> &str {
        &self.entity
    }

    fn find(&self, text: &str, out: &mut Vec<Finding>) {
        if self.regex.captures_len() > 1 {
            for caps in self.regex.captures_iter(text) {
                if let Some(m) = caps.get(1).or_else(|| caps.get(0)) {
                    self.push(text, m.start(), m.end(), out);
                }
            }
        } else {
            for m in self.regex.find_iter(text) {
                self.push(text, m.start(), m.end(), out);
            }
        }
    }
}
