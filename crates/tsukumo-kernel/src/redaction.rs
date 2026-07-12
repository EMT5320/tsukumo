//! Shared deterministic redaction policy for vendor and state boundaries.

use serde_json::Value;

const REDACTED: &str = "[REDACTED]";
const MAX_UNTRUSTED_JSON_ITEMS: usize = 64;
const MAX_UNTRUSTED_JSON_TEXT_CHARS: usize = 512;

/// Returns true when text resembles credential or secret material.
pub fn contains_sensitive_material(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    contains_private_key(&lower)
        || contains_secret_assignment(&lower)
        || contains_prefixed_token(&lower)
        || contains_high_entropy_token(text)
}

/// Removes terminal-unsafe characters and replaces secret-bearing text with one marker.
pub fn redact_sensitive_text(text: &str) -> String {
    let safe_controls = text
        .chars()
        .filter_map(|character| {
            if matches!(character, '\n' | '\r' | '\t') {
                Some(' ')
            } else if is_terminal_unsafe_character(character) {
                None
            } else {
                Some(character)
            }
        })
        .collect::<String>();
    if contains_sensitive_material(&safe_controls) {
        REDACTED.to_owned()
    } else {
        safe_controls
    }
}

/// Returns true for controls or invisible directional format characters unsafe in terminals.
pub fn is_terminal_unsafe_character(character: char) -> bool {
    character.is_control() || is_terminal_format_character(character)
}

fn is_terminal_format_character(character: char) -> bool {
    matches!(
        character,
        '\u{061c}'
            | '\u{200b}'
            | '\u{200c}'
            | '\u{200d}'
            | '\u{200e}'
            | '\u{200f}'
            | '\u{202a}'..='\u{202e}'
            | '\u{2060}'..='\u{2069}'
            | '\u{feff}'
    )
}
/// Recursively redacts sensitive keys and bounds untrusted JSON collections.
pub fn sanitize_untrusted_json(value: &Value) -> Value {
    sanitize_json_at_depth(value, 0)
}

/// Detects unredacted sensitive fields or values inside reviewed JSON.
pub fn contains_unredacted_sensitive_json(value: &Value) -> bool {
    match value {
        Value::Object(map) => map.iter().any(|(key, value)| {
            if is_sensitive_key(key) {
                value.as_str() != Some(REDACTED)
            } else {
                contains_unredacted_sensitive_json(value)
            }
        }),
        Value::Array(items) => items.iter().any(contains_unredacted_sensitive_json),
        Value::String(text) => text != REDACTED && contains_sensitive_material(text),
        Value::Null | Value::Bool(_) | Value::Number(_) => false,
    }
}

fn sanitize_json_at_depth(value: &Value, depth: usize) -> Value {
    if depth >= 32 {
        return Value::String("[TRUNCATED]".into());
    }
    match value {
        Value::Object(map) => Value::Object(
            map.iter()
                .take(MAX_UNTRUSTED_JSON_ITEMS)
                .map(|(key, value)| {
                    let value = if is_sensitive_key(key) {
                        Value::String(REDACTED.into())
                    } else {
                        sanitize_json_at_depth(value, depth + 1)
                    };
                    (bounded_text(key), value)
                })
                .collect(),
        ),
        Value::Array(items) => Value::Array(
            items
                .iter()
                .take(MAX_UNTRUSTED_JSON_ITEMS)
                .map(|item| sanitize_json_at_depth(item, depth + 1))
                .collect(),
        ),
        Value::String(text) => Value::String(bounded_text(&redact_sensitive_text(text))),
        other => other.clone(),
    }
}

fn bounded_text(text: &str) -> String {
    if text.chars().count() <= MAX_UNTRUSTED_JSON_TEXT_CHARS {
        return text.to_owned();
    }
    let mut prefix = text
        .chars()
        .take(MAX_UNTRUSTED_JSON_TEXT_CHARS - 1)
        .collect::<String>();
    prefix.push('…');
    prefix
}

fn is_sensitive_key(key: &str) -> bool {
    let normalized = key
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect::<String>();
    matches!(
        normalized.as_str(),
        "password"
            | "passwd"
            | "token"
            | "accesstoken"
            | "refreshtoken"
            | "idtoken"
            | "authtoken"
            | "apikey"
            | "apitoken"
            | "clientsecret"
            | "sessiontoken"
            | "credential"
            | "credentials"
            | "secretkey"
            | "signingkey"
            | "secret"
            | "privatekey"
            | "authorization"
            | "cookie"
            | "setcookie"
            | "awssecretaccesskey"
    )
}

fn contains_private_key(lower: &str) -> bool {
    lower.contains("-----begin ") && lower.contains("private key-----")
}

fn contains_secret_assignment(lower: &str) -> bool {
    const FIELDS: [&str; 14] = [
        "password",
        "passwd",
        "api_key",
        "api-key",
        "apikey",
        "access_token",
        "refresh_token",
        "client_secret",
        "private_key",
        "authorization",
        "cookie",
        "aws_secret_access_key",
        "secret",
        "token",
    ];
    FIELDS.iter().any(|field| {
        lower.match_indices(field).any(|(start, _)| {
            let suffix = &lower[start + field.len()..];
            let suffix = suffix.trim_start_matches(|character: char| {
                character.is_ascii_whitespace() || matches!(character, '"' | '\'' | ']')
            });
            let Some(separator) = suffix.chars().next() else {
                return false;
            };
            if !matches!(separator, ':' | '=') {
                return false;
            }
            let value = suffix[separator.len_utf8()..].trim_start_matches(|character: char| {
                character.is_ascii_whitespace() || matches!(character, '"' | '\'')
            });
            !value.is_empty() && !value.starts_with("[redacted]")
        })
    })
}

fn contains_prefixed_token(lower: &str) -> bool {
    [
        ("ghp_", 16),
        ("github_pat_", 16),
        ("sk-", 16),
        ("xoxb-", 16),
        ("xoxp-", 16),
        ("npm_", 16),
        ("akia", 16),
    ]
    .iter()
    .any(|(prefix, minimum_tail)| contains_long_token(lower, prefix, *minimum_tail))
}

fn contains_long_token(text: &str, prefix: &str, minimum_tail: usize) -> bool {
    text.match_indices(prefix).any(|(start, _)| {
        text[start + prefix.len()..]
            .chars()
            .take_while(|character| {
                character.is_ascii_alphanumeric() || matches!(character, '_' | '-' | '/' | '+')
            })
            .count()
            >= minimum_tail
    })
}

fn contains_high_entropy_token(text: &str) -> bool {
    text.split(|character: char| {
        !(character.is_ascii_alphanumeric() || matches!(character, '_' | '-' | '/' | '+' | '='))
    })
    .any(|token| {
        let length = token.len();
        if length < 32 {
            return false;
        }
        let has_lower = token.bytes().any(|byte| byte.is_ascii_lowercase());
        let has_upper = token.bytes().any(|byte| byte.is_ascii_uppercase());
        let has_digit = token.bytes().any(|byte| byte.is_ascii_digit());
        let has_symbol = token
            .bytes()
            .any(|byte| matches!(byte, b'_' | b'-' | b'/' | b'+' | b'='));
        has_lower && has_upper && has_digit && (has_symbol || length >= 40)
    })
}
