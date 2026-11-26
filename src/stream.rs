//! 流式还原：LLM 响应按 SSE chunk 到达时，占位符可能被切开（`<EMA` + `IL_1>`）。
//! `StreamRestorer` 只扣住末尾"可能是占位符前缀"的一小段，其余立即还原输出。

use crate::redact::restore;
use crate::vault::Vault;

pub struct StreamRestorer<'a> {
    vault: &'a Vault,
    pending: String,
}

impl<'a> StreamRestorer<'a> {
    pub fn new(vault: &'a Vault) -> Self {
        Self {
            vault,
            pending: String::new(),
        }
    }

    /// 喂入一个 chunk，返回可以安全输出的已还原文本。
    /// 所有 `push` 的输出拼上 `finish` 的输出，等于对完整文本调用 `restore`。
    pub fn push(&mut self, chunk: &str) -> String {
        self.pending.push_str(chunk);
        let hold = self.hold_from();
        let tail = self.pending.split_off(hold);
        let out = restore(&self.pending, self.vault);
        self.pending = tail;
        out
    }

    /// 流结束：还原并输出扣住的剩余部分。
    pub fn finish(self) -> String {
        restore(&self.pending, self.vault)
    }

    /// 需要扣住的起点：最后一个 `<` 起的尾巴恰好是某个已知占位符的真前缀时才扣住。
    fn hold_from(&self) -> usize {
        let p = &self.pending;
        match p.rfind('<') {
            Some(i) if self.is_placeholder_prefix(&p[i..]) => i,
            _ => p.len(),
        }
    }

    fn is_placeholder_prefix(&self, tail: &str) -> bool {
        self.vault
            .entries()
            .iter()
            .any(|e| e.placeholder.len() > tail.len() && e.placeholder.starts_with(tail))
    }
}
