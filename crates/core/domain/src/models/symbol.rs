use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Symbol {
    pub base: String,
    pub quote: String,
}

impl Symbol {
    pub fn new(base: impl Into<String>, quote: impl Into<String>) -> Self {
        Self {
            base: base.into().to_uppercase(),
            quote: quote.into().to_uppercase(),
        }
    }

    pub fn from_symbol_str(symbol: &str) -> Option<Self> {
        let clean = symbol.replace(['/', '_', '-'], "").to_uppercase();
        if clean.len() == 6 {
            Some(Self {
                base: clean[0..3].to_string(),
                quote: clean[3..6].to_string(),
            })
        } else {
            None
        }
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
