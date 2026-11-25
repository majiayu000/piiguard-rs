use std::fmt;

/// 构建 `Analyzer` 或加载配置时的错误。
#[derive(Debug)]
pub enum Error {
    UnknownEntity(String),
    InvalidPattern(regex::Error),
    #[cfg(feature = "serde")]
    Config(serde_json::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownEntity(e) => write!(f, "unknown builtin entity: {e}"),
            Self::InvalidPattern(e) => write!(f, "invalid pattern: {e}"),
            #[cfg(feature = "serde")]
            Self::Config(e) => write!(f, "invalid config: {e}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::UnknownEntity(_) => None,
            Self::InvalidPattern(e) => Some(e),
            #[cfg(feature = "serde")]
            Self::Config(e) => Some(e),
        }
    }
}

impl From<regex::Error> for Error {
    fn from(e: regex::Error) -> Self {
        Self::InvalidPattern(e)
    }
}
