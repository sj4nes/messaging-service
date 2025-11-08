/// Normalize phone numbers: keep leading '+' if present, retain digits only.
pub fn normalize_phone(raw: &str) -> String {
    let mut out = String::new();
    for (i, ch) in raw.chars().enumerate() {
        if ch.is_ascii_digit() {
            out.push(ch);
        } else if ch == '+' && i == 0 {
            out.push(ch);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_leading_plus_and_digits() {
        assert_eq!(normalize_phone("+1 (555) 000-1234"), "+15550001234");
    }

    #[test]
    fn strips_formatting_no_plus() {
        assert_eq!(normalize_phone("(555) 000-1234"), "5550001234");
    }
}
