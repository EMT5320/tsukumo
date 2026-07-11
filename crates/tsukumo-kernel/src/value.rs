//! Shared timestamp and persisted values used at durable boundaries.

use crate::redaction::{redact_sensitive_text, sanitize_untrusted_json};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt;

/// UTC Unix timestamp in milliseconds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Timestamp(i64);

impl Timestamp {
    /// Creates a timestamp from Unix milliseconds.
    pub const fn from_unix_millis(value: i64) -> Self {
        Self(value)
    }

    /// Returns the Unix millisecond representation.
    pub const fn as_unix_millis(self) -> i64 {
        self.0
    }
}

/// Secret-bearing text that must never serialize or appear in diagnostics.
#[derive(Clone, PartialEq, Eq)]
pub struct SensitiveText(String);

impl SensitiveText {
    /// Wraps secret-bearing text for an in-memory boundary.
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Exposes the value only at a validated storage or runtime boundary.
    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for SensitiveText {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("SensitiveText([REDACTED])")
    }
}

impl fmt::Display for SensitiveText {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("[REDACTED]")
    }
}

/// Text explicitly reviewed for durable event or state persistence.
#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PersistedText(String);

impl PersistedText {
    /// Marks text as reviewed by the owning persistence boundary.
    pub fn from_reviewed(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Redacts untrusted text before it crosses a durable boundary.
    pub fn from_redacted(value: impl AsRef<str>) -> Self {
        Self(redact_sensitive_text(value.as_ref()))
    }

    /// Returns the reviewed text.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for PersistedText {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("PersistedText([REDACTED])")
    }
}

impl fmt::Display for PersistedText {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

/// JSON reviewed for persistence with redacted diagnostic formatting.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PersistedJson(Value);

impl PersistedJson {
    /// Marks JSON as reviewed by the owning boundary.
    pub fn from_reviewed(value: Value) -> Self {
        Self(value)
    }

    /// Recursively bounds and redacts untrusted vendor JSON.
    pub fn from_untrusted(value: &Value) -> Self {
        Self(sanitize_untrusted_json(value))
    }

    pub fn as_value(&self) -> &Value {
        &self.0
    }
}

impl fmt::Debug for PersistedJson {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("PersistedJson([REDACTED])")
    }
}
