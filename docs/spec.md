# Spec：piiguard-rs（暂定名）—— 面向 LLM 网关热路径的 PII 检测与可逆脱敏

> 状态：草案，待确认。仓库名、crate 名都未在 crates.io 查重。

## 1. 为什么做

【事实】
- Python 标杆 presidio 有 12.1k★；Rust 移植 presidio-analyzer 约 4.4k 下载，2026-07 之后没有更新（子 agent 查 crates.io 所得）。
- litellm-rs 已有 `src/core/guardrails/pii.rs`（约 17KB），全是正则：
  - `BankAccount` 是 `\b\d{8,17}\b`、`SSN` 是 `\b\d{3}[-\s]?\d{2}[-\s]?\d{4}\b`，**没有校验位**，误报会很多
  - `Name`、`Address` 直接返回 `None`（没实现）
  - 没有中国大陆常见证件/号码
  - 只能单向打码，不能在响应里还原

【推断】网关场景最缺两样：**低误报**（靠校验位 + 上下文词）和**可逆脱敏**（请求里把 PII 换成占位符，响应回来再换回去，模型看不到真实值）。

## 2. 范围

### 做
1. 识别器（Recognizer）= 正则 + 校验函数 + 可选上下文词（附近出现"卡号/card"等才提高置信度）
   - 通用：邮箱、IPv4/IPv6、信用卡（Luhn）、IBAN（mod-97）、URL 中的 token 参数
   - 美国：SSN（排除 000/666/9xx 等无效段）、电话
   - 中国大陆：18 位身份证（GB 11643 校验位）、手机号（号段）、统一社会信用代码（GB 32100 校验位）、银行卡（Luhn + 长度）
   - 密钥类：常见 API key 前缀（sk-、ghp_、AKIA…）——与 keyspoor 的规则共享，见第 5 节
2. 每个命中给出 `score`（0–1），调用方设阈值
3. 三种处理：`Mask`（打码）、`Replace`（类型占位符 `<EMAIL_1>`）、`Pseudonymize`（可逆，映射表存在 `Vault` 里）
4. 多语言文本、UTF-8 字节偏移与字符偏移都给
5. 允许列表 / 拒绝列表 / 自定义识别器（配置文件加载）
6. CLI：`piiguard scan|redact|restore`，输入 stdin 或文件，输出 JSON 报告
7. 基准：单核吞吐（MB/s）与 p99 延迟，和 presidio 用同一份语料对比

### 不做（本期）
- 不做 NER（人名、地址），留 trait 扩展点，v0.2 再评估 ONNX 模型
- 不做图片 / PDF 中的 PII
- Vault 只做内存实现，不做持久化 / 分布式
- 不做合规认证声明（不写"符合 GDPR/个保法"之类）

## 3. 公开接口（草案）

```rust
pub trait Recognizer: Send + Sync {
    fn entity(&self) -> &str;                       // "CN_ID_CARD"
    fn find(&self, text: &str, out: &mut Vec<Finding>);
}

pub struct Finding { pub entity: String, pub start: usize, pub end: usize, pub score: f32 }

pub struct Analyzer { /* 已编译的 RegexSet + 识别器 */ }
impl Analyzer {
    pub fn builder() -> AnalyzerBuilder;            // .with_builtin(&["EMAIL","CN_ID_CARD"]).threshold(0.6)
    pub fn analyze(&self, text: &str) -> Vec<Finding>;   // 去重叠，按位置排序
}

pub enum Action { Mask { ch: char }, Replace, Pseudonymize }

pub struct Vault { /* placeholder <-> 原值 */ }
pub fn redact(text: &str, findings: &[Finding], action: &Action, vault: Option<&mut Vault>) -> String;
pub fn restore(text: &str, vault: &Vault) -> String;
```

不依赖 tokio / async；litellm-rs 侧自己包一层 `Guardrail` trait 实现。

## 4. 依赖

| crate | 用途 |
|---|---|
| regex（RegexSet） | 一次扫描匹配所有模式 |
| serde / serde_json | 配置与报告（feature `serde`） |
| clap | 仅 CLI |

## 5. 对现有仓库的影响（需要你确认）

- **litellm-rs**：`src/core/guardrails/pii.rs` 改为调用 piiguard-rs，删除其中的正则表与 mask 逻辑（不保留旧实现）。
  `PIIType` 枚举改为映射到 piiguard 的 entity 名，配置字段可能变化——这是破坏性改动，需要你同意。
  另外 `src/core/audit/redaction.rs`、`src/core/observability/redaction.rs` 也在做脱敏，迁移时一起评估能否复用。
- **keyspoor**：密钥识别规则目前在 keyspoor 里。两个选项：(a) piiguard 依赖 keyspoor 的规则 crate；(b) 本期不做密钥类识别。默认建议 (b)，避免一开始就跨仓库耦合。

## 6. 目录结构（每个文件 < 200 行）

```
src/lib.rs
src/analyzer.rs        RegexSet 编排、去重叠
src/recognizers/       email.rs  ip.rs  credit_card.rs  iban.rs  us_ssn.rs  cn_id.rs  cn_phone.rs  cn_uscc.rs  bank_card.rs
src/checksum.rs        Luhn / mod-97 / GB 11643 / GB 32100
src/redact.rs          Mask / Replace / Pseudonymize
src/vault.rs
src/bin/piiguard.rs
tests/                 每个识别器：合法样本、校验位错误样本、相似但非 PII 样本（订单号、时间戳）
benches/
```

## 7. 验收标准

1. `cargo test` 全绿；每个识别器都有"校验位错误必须不命中"的测试
2. `restore(redact(x)) == x`（Pseudonymize 模式）的属性测试
3. 测试样本全部是按算法生成的合成号码，不提交任何真实个人信息
4. 误报率：在一份不含 PII 的代码 + 日志语料上报告命中数；吞吐与 presidio 对比只写可复现的命令与结果
5. litellm-rs 接入后，原有 guardrails 测试全部通过

## 8. 待你决定

- crate / 仓库名（piiguard-rs 只是占位）
- litellm-rs 是否切换到新 crate（第 5 节）
- 密钥类识别本期做不做
- 仓库建在哪里、是否公开
