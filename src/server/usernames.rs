//! Stable, human-friendly usernames for every account.
//!
//! Anonymous accounts have no usable email, so friends add them by username.
//! Deriving the name from the numeric id keeps it stable without a schema
//! change and guarantees uniqueness through the id suffix:
//! `calm-otter-42`. The word lists are deliberately small and safe; the id
//! supplies the uniqueness, not the words.

const ADJECTIVES: &[&str] = &[
    "amber", "brave", "bright", "calm", "clever", "coral", "cosmic", "crisp", "gentle", "golden",
    "green", "happy", "hidden", "ivory", "kind", "lively", "lucky", "mellow", "misty", "noble",
    "olive", "quiet", "rapid", "silver", "sunny", "swift", "tender", "velvet", "violet", "warm",
    "wild", "young",
];

const NOUNS: &[&str] = &[
    "badger", "beacon", "birch", "breeze", "cedar", "cloud", "comet", "coral", "crane", "dawn",
    "dolphin", "ember", "falcon", "fern", "forest", "fox", "harbor", "heron", "island", "lantern",
    "maple", "meadow", "orbit", "otter", "pebble", "pine", "river", "robin", "sparrow", "star",
    "tide", "willow",
];

/// Stable username for an account. `id` is unique, so the suffix guarantees
/// no collisions even when two accounts pick the same adjective/noun pair.
pub fn username_for(id: i64) -> String {
    let n = id.unsigned_abs() as usize;
    let adjective = ADJECTIVES[n % ADJECTIVES.len()];
    let noun = NOUNS[(n / ADJECTIVES.len()) % NOUNS.len()];
    format!("{adjective}-{noun}-{n}")
}

/// Case-insensitive exact lookup; no substring/enumeration queries.
pub fn find_user_by_username<'a>(
    users: &'a [crate::models::User],
    username: &str,
) -> Option<&'a crate::models::User> {
    let wanted = username.trim().trim_start_matches('@').to_ascii_lowercase();
    if wanted.is_empty() {
        return None;
    }
    users
        .iter()
        .find(|u| username_for(u.id).eq_ignore_ascii_case(&wanted))
}

/// Public card: username + generated display name only. Never email, legal
/// name, numeric id or auth secrets.
pub fn public_json(user: &crate::models::User) -> serde_json::Value {
    let username = username_for(user.id);
    serde_json::json!({
        "username": username,
        "display_name": username,
        "auth_method": user.auth_method,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn usernames_are_stable_and_unique_by_id() {
        assert_eq!(username_for(42), username_for(42));
        assert_ne!(username_for(42), username_for(43));
        assert!(username_for(42).ends_with("-42"));
    }

    #[test]
    fn lookup_is_exact_and_case_insensitive() {
        use crate::models::User;
        let users = vec![User {
            id: 7,
            username: String::new(),
            email: "anon-7@device.pinpoint.invalid".into(),
            full_name: "Anonymous".into(),
            role: "resident".into(),
            is_active: true,
            must_change_password: false,
            password_hash: None,
            totp_secret: None,
            mfa_enabled: false,
            mfa_last_step: 0,
            failed_login_attempts: 0,
            locked_until: None,
            auth_method: "anonymous".into(),
            public_key: None,
            recovery_code_hash: None,
            identity_subject: None,
        }];
        let name = username_for(7);
        assert!(find_user_by_username(&users, &name).is_some());
        assert!(find_user_by_username(&users, &name.to_uppercase()).is_some());
        assert!(find_user_by_username(&users, "calm").is_none());
    }
}
