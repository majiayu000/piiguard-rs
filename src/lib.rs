//! piiguard：带校验位与上下文词的 PII 识别，以及可逆脱敏（占位符 + `Vault`）。
//!
//! ```
//! use piiguard::{Action, Analyzer, Vault, redact, restore};
//!
//! let analyzer = Analyzer::builder().with_builtin(&["EMAIL"]).build().unwrap();
//! let text = "联系 alice@example.com";
//! let findings = analyzer.analyze(text);
//! let mut vault = Vault::new();
//! let masked = redact(text, &findings, Action::Pseudonymize(&mut vault));
//! assert_eq!(masked, "联系 <EMAIL_1>");
//! assert_eq!(restore(&masked, &vault), text);
//! ```

mod analyzer;
pub mod checksum;
#[cfg(feature = "serde")]
mod config;
mod error;
mod recognizers;
mod redact;
mod stream;
mod vault;

pub use analyzer::{Analyzer, AnalyzerBuilder, DEFAULT_THRESHOLD};
#[cfg(feature = "serde")]
pub use config::{Config, CustomRule, DenyRule};
pub use error::Error;
pub use recognizers::{BUILTIN_ENTITIES, Finding, PatternRecognizer, Recognizer};
pub use redact::{Action, redact, restore};
pub use stream::StreamRestorer;
pub use vault::{Vault, VaultEntry};
