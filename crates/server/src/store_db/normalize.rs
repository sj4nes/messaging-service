use crate::queue::inbound_events::InboundEvent;

/// Normalize phone number: keep leading '+' then digits only.
fn normalize_phone(s: &str) -> String {
    let mut out = String::new();
    for c in s.chars() {
        if (c == '+' && out.is_empty()) || c.is_ascii_digit() {
            out.push(c);
        }
    }
    out
}

/// Normalize email: lowercase.
fn normalize_email(s: &str) -> String {
    s.to_ascii_lowercase()
}

pub fn normalize_addr(channel: &str, value: &str) -> String {
    match channel {
        "sms" | "mms" => normalize_phone(value),
        "email" => normalize_email(value),
        _ => value.to_string(),
    }
}

/// Build conversation key (channel + sorted normalized endpoints)
pub fn conversation_key(channel: &str, from: &str, to: &str) -> String {
    let nf = normalize_addr(channel, from);
    let nt = normalize_addr(channel, to);
    let (a, b) = if nf <= nt { (nf, nt) } else { (nt, nf) };
    format!("{}:{}<->{}", channel, a, b)
}

pub fn from_event(evt: &InboundEvent) -> (String, String, String) {
    // Expect metadata in payload for provider mock; fallback to event_type if absent
    let channel = evt.event_name.clone();
    // placeholder: actual integration will parse provider payload
    let from = "unknown_from".to_string();
    let to = "unknown_to".to_string();
    (channel, from, to)
}
