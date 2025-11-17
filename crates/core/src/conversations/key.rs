use super::{normalize_email::normalize_email, normalize_phone::normalize_phone, ConversationKey};

/// Supported channels for normalization
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChannelKind {
    Email,
    Sms,
    Mms,
}

fn normalize(channel: &ChannelKind, addr: &str) -> String {
    match channel {
        ChannelKind::Email => normalize_email(addr),
        ChannelKind::Sms | ChannelKind::Mms => normalize_phone(addr),
    }
}

/// Build canonical conversation key and ordered participants
pub fn derive_key(channel: ChannelKind, a: &str, b: &str) -> ConversationKey {
    let na = normalize(&channel, a);
    let nb = normalize(&channel, b);
    let (pa, pb) = if na <= nb { (na, nb) } else { (nb, na) };
    let chan = match channel {
        ChannelKind::Email => "email",
        ChannelKind::Sms => "sms",
        ChannelKind::Mms => "mms",
    }
    .to_string();
    let key = format!("{}:{}<->{}", chan, pa, pb);
    ConversationKey {
        channel: chan,
        participant_a: pa,
        participant_b: pb,
        key,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn orders_participants() {
        let k = derive_key(ChannelKind::Email, "B@example.com", "a@example.com");
        assert_eq!(k.participant_a, "a@example.com");
        assert_eq!(k.participant_b, "b@example.com");
        assert_eq!(k.key, "email:a@example.com<->b@example.com");
    }

    #[test]
    fn normalizes_phone_digits() {
        let k = derive_key(ChannelKind::Sms, "+1 (555) 000-1234", "5550001234");
        assert_eq!(k.participant_a, "+15550001234");
        assert_eq!(k.participant_b, "+15550001234");
        assert_eq!(k.key, "sms:+15550001234<->+15550001234");
    }
}
