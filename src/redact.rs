//! 三种处理方式与还原。

use std::collections::HashMap;

use crate::recognizers::Finding;
use crate::vault::{Vault, placeholder};

pub enum Action<'a> {
    /// 每个字符替换成 `ch`，字符数不变。
    Mask { ch: char },
    /// 替换为类型占位符 `<EMAIL_1>`，同一值在一次调用内编号相同；不可还原。
    Replace,
    /// 替换为占位符并把映射写进 `Vault`，之后可用 `restore` 还原。
    Pseudonymize(&'a mut Vault),
}

/// 按 `findings` 处理文本。`findings` 应来自 `Analyzer::analyze`（已排序、无重叠）；
/// 与前一个命中重叠或不在字符边界上的命中会被跳过。
pub fn redact(text: &str, findings: &[Finding], action: Action<'_>) -> String {
    let mut action = action;
    let mut local: HashMap<(&str, &str), usize> = HashMap::new();
    let mut counters: HashMap<&str, usize> = HashMap::new();
    let mut out = String::with_capacity(text.len());
    let mut cursor = 0;
    for f in findings {
        let valid = f.start >= cursor
            && f.start < f.end
            && f.end <= text.len()
            && text.is_char_boundary(f.start)
            && text.is_char_boundary(f.end);
        if !valid {
            continue;
        }
        out.push_str(&text[cursor..f.start]);
        let value = &text[f.start..f.end];
        match &mut action {
            Action::Mask { ch } => out.extend(std::iter::repeat_n(*ch, value.chars().count())),
            Action::Replace => {
                let n = *local.entry((f.entity.as_str(), value)).or_insert_with(|| {
                    let c = counters.entry(f.entity.as_str()).or_insert(0);
                    *c += 1;
                    *c
                });
                out.push_str(&placeholder(&f.entity, n));
            }
            Action::Pseudonymize(vault) => out.push_str(&vault.pseudonym(&f.entity, value, text)),
        }
        cursor = f.end;
    }
    out.push_str(&text[cursor..]);
    out
}

/// 把 `vault` 中已知的占位符换回原值；未知的 `<...>` 原样保留。
pub fn restore(text: &str, vault: &Vault) -> String {
    let max = vault.max_placeholder_len();
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(lt) = rest.find('<') {
        out.push_str(&rest[..lt]);
        let tail = &rest[lt..];
        let hit = tail
            .char_indices()
            .take_while(|(i, _)| *i < max)
            .find(|(_, c)| *c == '>')
            .and_then(|(gt, _)| vault.original(&tail[..=gt]).map(|v| (gt, v)));
        match hit {
            Some((gt, v)) => {
                out.push_str(v);
                rest = &tail[gt + 1..];
            }
            None => {
                out.push('<');
                rest = &tail[1..];
            }
        }
    }
    out.push_str(rest);
    out
}
