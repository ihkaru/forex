use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Symbol {
    pub base: String,
    pub quote: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub raw: Option<String>,
}

impl PartialEq for Symbol {
    fn eq(&self, other: &Self) -> bool {
        self.base == other.base && self.quote == other.quote
    }
}

impl Eq for Symbol {}

impl std::hash::Hash for Symbol {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.base.hash(state);
        self.quote.hash(state);
    }
}

impl Symbol {
    pub fn new(base: impl Into<String>, quote: impl Into<String>) -> Self {
        Self {
            base: base.into().to_uppercase(),
            quote: quote.into().to_uppercase(),
            raw: None,
        }
    }

    pub fn with_raw(mut self, raw: impl Into<String>) -> Self {
        self.raw = Some(raw.into());
        self
    }

    pub fn from_symbol_str(symbol: &str) -> Option<Self> {
        let clean = symbol.replace(['/', '-'], "").to_uppercase();
        let s_clean = clean.replace('_', "");
        let raw_upper = symbol.trim().to_uppercase();

        if s_clean.len() >= 6 {
            Some(Self {
                base: s_clean[0..3].to_string(),
                quote: s_clean[3..6].to_string(),
                raw: Some(raw_upper),
            })
        } else {
            None
        }
    }

    pub fn raw_ticker(&self) -> String {
        self.raw.clone().unwrap_or_else(|| self.to_compact_string())
    }

    pub fn to_pair_string(&self) -> String {
        format!("{}/{}", self.base, self.quote)
    }

    pub fn to_compact_string(&self) -> String {
        format!("{}{}", self.base, self.quote)
    }
}

impl std::str::FromStr for Symbol {
    type Err = crate::errors::DomainError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_symbol_str(s)
            .ok_or_else(|| crate::errors::DomainError::InvalidSymbol(s.to_string()))
    }
}

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.base, self.quote)
    }
}
