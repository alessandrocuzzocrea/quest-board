const BASE62: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

/// Generate a short URL-safe slug (11 chars, base62, ~66 bits of entropy).
/// Generated from a random UUID so no extra randomness source needed.
pub fn generate_slug() -> String {
    let uuid = uuid::Uuid::new_v4();
    let n = uuid.as_u128();
    let mut slug = String::with_capacity(11);
    let mut remaining = n;
    for _ in 0..11 {
        slug.push(BASE62[(remaining % 62) as usize] as char);
        remaining /= 62;
    }
    slug
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slug_length_and_chars() {
        let slug = generate_slug();
        assert_eq!(slug.len(), 11);
        assert!(slug.chars().all(|c| c.is_ascii_alphanumeric()));
    }

    #[test]
    fn slug_unique() {
        let slugs: std::collections::HashSet<String> =
            (0..1000).map(|_| generate_slug()).collect();
        assert_eq!(slugs.len(), 1000);
    }
    #[test]
    fn slug_not_hex_style() {
        // Slug should look YouTube-like, not a hex UUID fragment.
        // At minimum it must contain letters outside 0-9a-f range.
        let slug = generate_slug();
        let has_wide_chars = slug.chars().any(|c| c > 'f' || (c > '9' && c < 'a'));
        assert!(has_wide_chars, "slug {:?} looks like hex, expected base62 YouTube-style", slug);
    }
}
