//! Input validation and security helpers for web clients.
//!
//! Client-side validation is a UX and defense-in-depth measure only; the backend
//! remains authoritative and must enforce all security constraints (fail closed).

/// Maximum lengths to prevent abuse/resource exhaustion in client-side handling.
pub const MAX_TOKEN_LEN: usize = 512;
pub const MAX_NOTE_LEN: usize = 500;
pub const MAX_QUERY_LEN: usize = 200;
pub const MAX_EMAIL_LEN: usize = 254;

/// Redact potentially sensitive values for logs/snapshots.
pub fn redact_secret(value: &str) -> String {
    let len = value.chars().count();
    if len == 0 {
        return "<empty>".into();
    }
    if len <= 4 {
        return "***".into();
    }
    format!("***({len} chars)")
}

/// Validate an email-like identifier for client UX (basic shape, not RFC-complete).
/// Backend performs canonical validation.
pub fn validate_email_basic(email: &str) -> Result<(), String> {
    let e = email.trim();
    if e.is_empty() {
        return Err("Email is required.".into());
    }
    if e.len() > MAX_EMAIL_LEN {
        return Err("Email is too long.".into());
    }
    if e.chars().any(|c| c.is_control() || c.is_whitespace()) {
        return Err("Email contains invalid characters.".into());
    }
    let Some((local, domain)) = e.split_once('@') else {
        return Err("Email must contain '@'.".into());
    };
    if local.is_empty() || domain.is_empty() || !domain.contains('.') {
        return Err("Email format is invalid.".into());
    }
    Ok(())
}

/// Validate a search/query string (client-side bounds/sanity).
pub fn validate_query(q: &str) -> Result<String, String> {
    let t = q.trim().to_string();
    if t.len() > MAX_QUERY_LEN {
        return Err("Search query is too long.".into());
    }
    if t.chars().any(|c| c == '\0' || c.is_control() && c != ' ') {
        return Err("Search query contains invalid characters.".into());
    }
    Ok(t)
}

/// Validate a kiosk note field.
pub fn validate_note(note: Option<&str>) -> Result<Option<String>, String> {
    let Some(n) = note else { return Ok(None) };
    let t = n.trim();
    if t.is_empty() {
        return Ok(None);
    }
    if t.chars().count() > MAX_NOTE_LEN {
        return Err("Note is too long.".into());
    }
    if t.chars().any(|c| c == '\0') {
        return Err("Note contains invalid characters.".into());
    }
    Ok(Some(t.to_string()))
}

/// Escape text for safe HTML rendering (XSS defense-in-depth in DOM-based UIs).
pub fn escape_html(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn email_basic_validation() {
        assert!(validate_email_basic("a@b.com").is_ok());
        assert!(validate_email_basic("nope").is_err());
        assert!(validate_email_basic("").is_err());
    }

    #[test]
    fn redact_does_not_leak_secret() {
        assert!(!redact_secret("supersecret").contains("supersecret"));
        assert!(redact_secret("supersecret").contains("chars"));
    }

    #[test]
    fn escape_html_neutralizes_markup() {
        assert_eq!(
            escape_html("<a>\"x\"</a>"),
            "&lt;a&gt;&quot;x&quot;&lt;/a&gt;"
        );
    }
}
