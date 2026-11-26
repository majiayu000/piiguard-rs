# 竞品功能对比（2026-11-26）

> 说明：本表只写**核实过**的事实。"出处"为本机安装的竞品源码路径（Python 包相对 `site-packages/`，Rust crate 相对 `~/.cargo/registry/src/*/`）或实际运行的命令；查不到的写"未核实"。
> 吞吐、延迟、准确率等数字不在本文件，见 [benchmark.md](benchmark.md)。

## 1. 参与对比的项目与版本

| 项目 | 语言 | 版本 | 定位 | 本次是否实跑 |
|---|---|---|---|---|
| piiguard（本仓库） | Rust | 0.1.0 | 网关热路径：校验位 + 可逆占位符 | 是 |
| Microsoft presidio-analyzer + presidio-anonymizer | Python | 2.2.366 | 通用 PII 检测与匿名化框架，规则 + NER | 是（两组配置） |
| presidio-analyzer（presidio-rust 移植） | Rust | 0.1.11 | presidio 的 Rust 移植 | 是 |
| pii-vault | Rust（另有 TS/Python/Go） | 0.2.0 | Presidio 兼容规则 + 可逆 token vault | 是（规则取自 git tag v0.2.0 的 `spec/recognizers`，crate 包不含） |
| pii-veil-core | Rust | 1.0.0 | 面向 LLM 的可逆脱敏 + 密钥识别 | 是 |
| cloakpipe-core | Rust | 0.15.0 | LLM 出口代理的伪名化引擎（规则 + 可选 ONNX NER） | 是（打开电话、IP 检测） |
| llm-pii-redact | Rust | 0.1.0 | 面向 LLM 提示的轻量正则脱敏 + 可逆占位符 | 是 |
| redacter | Rust（CLI） | 0.23.0 | 文件级检测/脱敏 CLI，本地规则或云 DLP/LLM | 是（`-d local-rules`） |
| scrubadub | Python | 2.0.1 | 文本 PII 清洗库 | 是 |

搜索方式：`cargo search pii`、`cargo search presidio`、`cargo search redacter`、PyPI JSON API。crates.io 上还有 `pii`、`pii-masker`、`sanitize-pii`、`newmean-pii`（54 语言小模型）、`patronus-ark` 等，**未核实、未纳入**。

## 2. 规则类功能

图例：有 / 无 / 部分。

| 功能 | piiguard | presidio (Py) | presidio-rs | pii-vault | pii-veil | cloakpipe | llm-pii-redact | redacter | scrubadub |
|---|---|---|---|---|---|---|---|---|---|
| 邮箱 | 有 | 有 | 有 | 有 | 有 | 有 | 有 | 有 | 有 |
| IPv4 / IPv6 | 有 / 有 | 有 / 有 | 有 / 有 | 有 / 未核实 | 有 / 有 | 有 / 无 | 有 / 有 | 有 / 有 | 无 |
| 信用卡 + Luhn | 有 | 有 | 有 | 有 | 有 | 无 | 有 | 有 | 有 |
| IBAN + mod-97 | 有 | 有 | 有 | 有 | 有 | 无 | 部分（只正则，未见 mod-97） | 有 | 无 |
| 美国 SSN（号段校验） | 有 | 有 | 有 | 有 | 有 | 部分（Custom 规则） | 部分（只正则） | 有 | 有 |
| 电话 | 美国 NANP + 中国号段 | 有（phonenumbers 库） | 有 | 有 | 有 | 有 | 美国 | 有 | 有（phonenumbers） |
| 中国身份证 + GB 11643 | 有（含区划码、日期） | 无 | 有（只校验位） | 有（`cn_id_checksum`） | 无 | 无 | 无 | 有 | 无 |
| 中国手机号（号段） | 有 | 无（通用电话识别可能命中） | 无专门规则 | 有（`cn_phone.json`） | 无 | 无 | 无 | 无专门规则 | 无 |
| 统一社会信用代码 + GB 32100 | 有 | 无 | 无 | 无 | 无 | 无 | 无 | 无 | 无 |
| 中国银行卡（Luhn + 长度） | 有 | 无 | 无 | 有（`cn_bank_card.json`） | 无 | 无 | 无 | 无 | 无 |
| URL 中的 token 参数 | 有 | 部分（整条 URL 识别） | 部分（整条 URL） | 未核实 | 部分（webhook/连接串） | 部分（URL） | 部分（URL） | 部分（URL 规则组） | 部分（URL） |
| 其他国家证件 | 无 | 有（约 75 个国家识别器，默认只启用一部分） | 有（约 50 种） | 有（47 个规则文件） | 有（约 12 种） | 部分（Aadhaar/PAN） | 无 | 有（EU/World 规则组） | 部分（英国 NINO 等） |
| 日期 / MAC / UUID / 加密货币地址 | 无 | 有 | 有 | 有 | 部分（MAC） | 部分（日期） | 无 | 部分 | 无 |
| 上下文词加分 | 有（命中前 32 字符；另有负向上下文：紧挨字段名时扣分，本期新增） | 有（词形 lemma，需 NLP 引擎） | 有 | 有 | 部分（启发式） | 未核实 | 无 | 未核实 | 无 |
| 分数 + 阈值 | 有 | 有 | 有 | 有 | 无（命中即报） | 有（confidence） | 无 | 部分（score 字段为 null） | 无 |
| 允许列表 / 拒绝列表 | 有 / 有 | 有 / 有 | 有 / 有 | 未核实 / 有 | 有 / 部分（自定义规则） | 未核实 | 无 / 无 | 未核实 | 未核实 |
| 自定义识别器 | trait + JSON 配置 | 代码 + YAML | 代码 | JSON 规则目录 | 代码 + 策略文件 | TOML 规则 | 代码（正则） | JSON 规则文件 / `name=regex` | 代码 |
| 一次扫描多模式（RegexSet 预筛） | 无（本期实测更慢后去掉，见 benchmark.md 第 5 节） | 无（逐识别器） | 未核实 | 无（逐识别器循环） | 未核实 | 未核实 | 无 | 未核实 | 无 |
| 字节偏移 / 字符偏移 | 都有 | 字符 | 字节 | 字节 | 字节 | 字节 | 字节 | 有 | 字符 |
| 处理：打码 / 类型占位 / 删除 / 哈希 / 加密 / 假数据 | 有 / 有 / 无 / 无 / 无 / 无 | 有 / 有 / 有 / 有 / 有（AES） / 部分（AHDS surrogate） | 未核实（在 presidio-anonymizer crate） | 有 / 有 / 有 / 有 / 无 / 无 | 有 / 有 / 无 / 无 / 无 / 有（Synthetic） | 未核实 / 有 / 未核实 / 未核实 / 无 / 有（format-preserving） | 无 / 有 / 无 / 无 / 无 / 无 | 有（文件级） | 无 / 有 / 有 / 有 / 无 / 无 |
| 可逆还原 | 有（Vault + `restore`） | 部分（只有 encrypt→decrypt） | 有（deanonymize，crate 描述） | 有（Vault `detokenize`） | 有 | 有（rehydrator） | 有（`reveal`） | 无 | 无 |
| **流式还原（SSE 占位符跨 chunk）** | 有（本期新增 `StreamRestorer`） | 无 | 未核实 | 无 | 有（`StreamingRestorer`） | 有（rehydrator 处理 SSE chunk） | 无 | 无 | 无 |
| Vault 持久化 / 加密 | JSON（CLI） / 无 | — | 未核实 | JSON / 无 | JSON / 有（认证加密） | SQLite / 有（AES-256-GCM） | 无 | — | — |
| 密钥 / API key 识别 | 无（本期约定不做） | 部分（第三方识别器） | 未核实 | 未核实 | 有（OpenAI/Anthropic/AWS/GitHub/Stripe/JWT/PEM 等） | 有 | 无 | 有（Secrets 规则组） | 部分（credential） |
| 中文文本稳定性（实测） | 不崩溃 | 不崩溃 | 不崩溃 | **会 panic**（见 benchmark.md） | 不崩溃 | 不崩溃 | 不崩溃 | 不崩溃 | 不崩溃 |
| CLI | 有 | 无（有 REST 镜像） | 有（presidio-cli crate） | 未核实 | 有（pii-veil-cli） | 有（代理） | 无 | 有 | 无 |
| HTTP 服务 | 无 | 有（Docker 镜像） | 有（presidio-server crate） | 未核实 | 未核实 | 有（cloakpipe-proxy） | 无 | 无 | 无 |
| 文件 / 图片 / PDF | 无 | 部分（presidio-image-redactor 另一包） | 未核实 | 无 | 部分（SQL/CSV/NDJSON 流） | 未核实 | 无 | 有（多种文件格式、OCR addon） | 无 |
| async / 运行时依赖 | 无 | — | 未核实 | 无 | 未核实 | 未核实 | 无 | tokio（CLI） | — |

### 出处

- presidio (Py)：`presidio_analyzer/predefined_recognizers/{generic,country_specific}/`（无 `china` 目录）、`presidio_analyzer/pattern_recognizer.py:29`（deny_list）、`presidio_analyzer/analyzer_engine.py:179`（allow_list）、`presidio_anonymizer/operators/`（replace/redact/hash/mask/encrypt/decrypt/custom/keep/ahds_surrogate）、`presidio_analyzer/nlp_engine/no_op_nlp_engine.py`（无 NLP 时上下文增强失效的说明）；默认启用的识别器列表由 `AnalyzerEngine().registry.recognizers` 实际打印。
- presidio-rs：`presidio-analyzer-0.1.11/src/predefined.rs`、`src/country.rs:854`（`cn_ric`，只校验位，基础分 0.1）、`src/analyzer_engine.rs:37`（allow_list）、`src/recognizer.rs:46`（deny_list）、`src/ner_recognizer.rs`（默认 SimpleNlpEngine 下 NER 不产出）；`cargo search presidio` 列出 presidio-anonymizer / presidio-server / presidio-cli / presidio-ner。
- pii-vault：`pii-vault-0.2.0/src/anonymizer.rs:7`（Operator 枚举）、`src/recognizer.rs:72-83`（validators）、`src/vault.rs`、`src/recognizer.rs:63`（panic 位置）、`spec/recognizers/`（git tag v0.2.0）。
- pii-veil：`pii-veil-core-1.0.0/src/lib.rs` 文档、`src/detector.rs:9-30`（EntityKind）、`src/detector.rs:819`（NATIONAL_ID 覆盖范围，含 US SSN、不含中国身份证）、`src/strategy.rs:7`（RedactionMode）、`src/vault.rs:186`（加密 vault）、`src/vault.rs:346`（StreamingRestorer）。
- cloakpipe：`cloakpipe-core-0.15.0/src/detector/patterns.rs`、`src/config.rs:114`（DetectionConfig，电话/IP 默认关闭）、`src/rehydrator.rs:3`、`src/vault.rs:4`、`src/vault_sqlite.rs`。
- llm-pii-redact：`llm-pii-redact-0.1.0/src/lib.rs:187`（默认检测器）、`:410`（reveal）。
- redacter：`redacter-0.23.0/src/redacters/local_rules/rules/mod.rs:19`（RuleGroup）；实际运行 `redacter scan -d local-rules --report-file r.json`。
- scrubadub：`scrubadub/detectors/`、`scrubadub/post_processors/filth_replacer.py:48`（include_hash）；默认检测器由 `scrubadub.Scrubber()._detectors` 实际打印。

## 3. 需要 ML / NER 模型的功能（单独比较）

| 项目 | 人名 / 地址 / 机构 | 实现方式 | 默认是否启用 |
|---|---|---|---|
| piiguard | 无 | 只留 `Recognizer` trait 扩展点 | — |
| presidio (Py) | 有 | `SpacyRecognizer` + spaCy `en_core_web_lg`（默认）；可换 Stanza、HuggingFace transformers、GLiNER（`predefined_recognizers/ner/`、`nlp_engine_recognizers/`）；另有 Azure AI Language / LangExtract 第三方识别器 | 是（默认 spaCy lg，约 400 MB 模型） |
| presidio-rs | 有（可选） | `presidio-ner` crate（Candle）、gazetteer 名单（cargo feature）、`onnx-pos` feature | 否（默认 SimpleNlpEngine 不产出 NER） |
| pii-veil | 部分 | `ner` feature 的 `HeuristicNerDetector`（英/西语人名地址启发式，无模型）或自接 ONNX/candle | 否 |
| cloakpipe | 有（可选） | ONNX BERT NER、GLiNER2、DistilBERT-PII、Nemotron-PII（`src/detector/`，`ner` feature 依赖 `ort`） | 否 |
| redacter | 有（可选） | `local-ner`、`local-gliner` 本地模型 addon，或 GCP DLP / AWS Comprehend / Gemini / OpenAI 等云服务 | 否 |
| scrubadub | 部分 | 可选 TextBlob / spaCy / Stanford 名字检测器（需额外安装） | 否 |
| pii-vault / llm-pii-redact | 无 | — | — |

结论【事实】：只有 Python presidio 默认就开着 NER。所有 Rust 项目要么没有 NER，要么放在可选 feature 里，代价是要带上 ONNX Runtime 或 Candle 加模型文件。

## 4. 本期补齐的差距（规则类）

| 差距 | 补齐方式 | 证据 |
|---|---|---|
| 吞吐落后 pii-veil、llm-pii-redact、cloakpipe | 去掉 `RegexSet` 预筛 | benchmark.md 第 5 节：labeled 1.68 → 19.8 MB/s，clean 12.77 → 51.74 MB/s |
| 流式还原（pii-veil、cloakpipe 有） | `StreamRestorer` | `tests/stream.rs`（含 proptest） |
| 四段版本号、订单号导致的 IP/卡号误报（所有竞品都有） | 负向上下文 | clean 误报 1545 → 10 |
| 银行卡无上下文时召回低 | 增加常见转账类上下文词 | CN_BANK_CARD R 0.346 → 0.998（有在同一语料上调参的偏差，见 benchmark.md 第 5 节风险） |

仍未补齐、但属于规则类的：clean 语料（日志 + 代码）上的吞吐 51.74 MB/s，仍低于 pii-veil 132.25、llm-pii-redact 67.14、cloakpipe 62.98。原因【推断，有剖析数据支撑】：11 个识别器各自全文扫描一遍，每遍约 400 MB/s，合计上限约 36–50 MB/s；要再提速，需要先用一遍廉价扫描找出数字候选区，再只在候选区上跑各识别器，这要改识别器架构，本期没做。

## 5. 仍落后、且需要用户决定的差距

| 差距 | 谁有 | 补齐代价 |
|---|---|---|
| 人名 / 地址 NER | presidio（默认）、cloakpipe/presidio-rs/redacter（可选） | 要引入 ONNX Runtime（`ort`，原生库约数十 MB）或 Candle 加模型（数百 MB），单篇延迟从微秒级升到毫秒级，与"热路径"定位冲突；建议做成独立可选 crate |
| 密钥 / API key 识别 | pii-veil、cloakpipe、redacter | 规则本身很轻，但与 keyspoor 的规则重复，需要先定共享方式（spec 第 5 节） |
| 其他国家证件（数十种） | presidio、presidio-rs、pii-vault | 每种都要写校验算法和测试，工作量线性增长；按目标用户地区挑选更合适 |
| 加密 Vault / 持久化 | pii-veil（认证加密）、cloakpipe（AES-GCM + SQLite） | 要引入 `aes-gcm` 等加密依赖和密钥管理接口；网关一般有自己的存储层 |
| 哈希 / 加密 / 假数据替换 | presidio、pii-vault、pii-veil、cloakpipe | 哈希、假数据不需要大依赖，可以按需加；加密需要加密依赖 |
| 文件 / 图片 / PDF | redacter、presidio-image-redactor | 不在网关文本热路径范围内 |
