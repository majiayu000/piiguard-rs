//! 内存映射表：占位符 `<ENTITY_n>` ↔ 原值。不做持久化；CLI 通过 serde 存成 JSON 文件。

use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct VaultEntry {
    pub placeholder: String,
    pub entity: String,
    pub value: String,
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(from = "Vec<VaultEntry>", into = "Vec<VaultEntry>")
)]
pub struct Vault {
    entries: Vec<VaultEntry>,
    by_value: HashMap<(String, String), usize>,
    by_placeholder: HashMap<String, usize>,
    counters: HashMap<String, usize>,
    max_placeholder_len: usize,
}

pub(crate) fn placeholder(entity: &str, n: usize) -> String {
    format!("<{entity}_{n}>")
}

impl Vault {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn entries(&self) -> &[VaultEntry] {
        &self.entries
    }

    /// 占位符对应的原值。
    pub fn original(&self, placeholder: &str) -> Option<&str> {
        self.by_placeholder
            .get(placeholder)
            .map(|&i| self.entries[i].value.as_str())
    }

    pub(crate) fn max_placeholder_len(&self) -> usize {
        self.max_placeholder_len
    }

    /// 同一 (entity, value) 始终得到同一占位符；新分配时跳过 `avoid` 里已出现的占位符，
    /// 保证原文中字面存在的 `<EMAIL_1>` 不会在还原时被误替换。
    pub(crate) fn pseudonym(&mut self, entity: &str, value: &str, avoid: &str) -> String {
        let key = (entity.to_string(), value.to_string());
        if let Some(&i) = self.by_value.get(&key) {
            return self.entries[i].placeholder.clone();
        }
        let counter = self.counters.entry(entity.to_string()).or_insert(0);
        let ph = loop {
            *counter += 1;
            let ph = placeholder(entity, *counter);
            if !avoid.contains(&ph) {
                break ph;
            }
        };
        self.insert(VaultEntry {
            placeholder: ph.clone(),
            entity: key.0,
            value: key.1,
        });
        ph
    }

    fn insert(&mut self, e: VaultEntry) {
        let n = e
            .placeholder
            .strip_suffix('>')
            .and_then(|p| p.rsplit_once('_'))
            .and_then(|(_, n)| n.parse::<usize>().ok())
            .unwrap_or(0);
        let counter = self.counters.entry(e.entity.clone()).or_insert(0);
        *counter = (*counter).max(n);
        self.max_placeholder_len = self.max_placeholder_len.max(e.placeholder.len());
        let i = self.entries.len();
        self.by_value.insert((e.entity.clone(), e.value.clone()), i);
        self.by_placeholder.insert(e.placeholder.clone(), i);
        self.entries.push(e);
    }
}

impl From<Vec<VaultEntry>> for Vault {
    fn from(entries: Vec<VaultEntry>) -> Self {
        let mut v = Vault::new();
        for e in entries {
            v.insert(e);
        }
        v
    }
}

impl From<Vault> for Vec<VaultEntry> {
    fn from(v: Vault) -> Self {
        v.entries
    }
}
