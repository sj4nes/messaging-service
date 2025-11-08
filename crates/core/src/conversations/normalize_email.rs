/// Normalize email addresses: lowercase entire address and strip plus-tags.
pub fn normalize_email(addr: &str) -> String {
    let lower = addr.to_ascii_lowercase();
    if let Some(at_pos) = lower.find('@') {
        let (local, domain) = lower.split_at(at_pos);
        let local_base = local.split('+').next().unwrap_or(local);
        format!("{}{}", local_base, domain)
    } else {
        lower
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lowers_case() {
        assert_eq!(normalize_email("USER@Example.COM"), "user@example.com");
    }

    #[test]
    fn strips_plus_tag() {
        assert_eq!(normalize_email("user+tag@example.com"), "user@example.com");
    }

    #[test]
    fn leaves_no_at() {
        assert_eq!(normalize_email("not-an-email"), "not-an-email");
    }
}
