use std::collections::HashMap;
use std::sync::{OnceLock, RwLock};

use crate::store::messages::{Channel, StoredMessage};
use crate::types::{ConversationDto, MessageDto};

#[derive(Debug, Clone)]
struct Conversation {
    id: String,
    key: String,
    message_count: u32,
    last_activity_at: String,
}

fn map() -> &'static RwLock<HashMap<String, Conversation>> {
    static CELL: OnceLock<RwLock<HashMap<String, Conversation>>> = OnceLock::new();
    CELL.get_or_init(|| RwLock::new(HashMap::new()))
}

fn normalize_addr(channel: &Channel, s: &str) -> String {
    match channel {
        Channel::Email => s.to_ascii_lowercase(),
        Channel::Sms | Channel::Mms => {
            let mut out = String::new();
            for c in s.chars() {
                if (c == '+' && out.is_empty()) || c.is_ascii_digit() {
                    out.push(c);
                }
            }
            out
        }
    }
}

fn convo_id(channel: &Channel, from: &str, to: &str) -> (String, String) {
    let nf = normalize_addr(channel, from);
    let nt = normalize_addr(channel, to);
    let (a, b) = if nf <= nt { (nf, nt) } else { (nt, nf) };
    let key = format!(
        "{}:{}<->{}",
        match channel {
            Channel::Sms => "sms",
            Channel::Mms => "mms",
            Channel::Email => "email",
        },
        a,
        b
    );
    let id = key.clone();
    (id, key)
}

pub fn on_message_stored(msg: &StoredMessage) {
    let (id, key) = convo_id(&msg.channel, &msg.from, &msg.to);
    let mut w = map().write().unwrap();
    let entry = w.entry(id.clone()).or_insert(Conversation {
        id,
        key,
        message_count: 0,
        last_activity_at: msg.timestamp.clone(),
    });
    entry.message_count = entry.message_count.saturating_add(1);
    if msg.timestamp > entry.last_activity_at {
        entry.last_activity_at = msg.timestamp.clone();
    }
}

pub fn list(page: u32, page_size: u32) -> (Vec<ConversationDto>, u64) {
    let r = map().read().unwrap();
    let mut items: Vec<_> = r.values().cloned().collect();
    // Sort by last_activity_at desc
    items.sort_by(|a, b| b.last_activity_at.cmp(&a.last_activity_at));
    let total = items.len() as u64;
    let ps = if page_size == 0 { 50 } else { page_size.min(50) } as usize;
    let start = ((page.max(1) - 1) * ps as u32) as usize;
    let end = (start + ps).min(items.len());
    let slice = if start < end { &items[start..end] } else { &[] };
    let dtos = slice
        .iter()
        .map(|c| ConversationDto {
            id: c.id.clone(),
            key: c.key.clone(),
            channel: None,
            participant_a: None,
            participant_b: None,
            message_count: c.message_count,
            last_activity_at: c.last_activity_at.clone(),
        })
        .collect();
    (dtos, total)
}

pub fn list_messages(
    conv_id: &str,
    page: u32,
    page_size: u32,
    snippet_len: usize,
) -> (Vec<MessageDto>, u64) {
    // For simplicity, derive messages by scanning the message store and selecting matching convo
    let all = crate::store::messages::all();
    // Filter by conversation id
    let mut msgs: Vec<_> = all
        .into_iter()
        .filter(|m| {
            let (id, _) = convo_id(&m.channel, &m.from, &m.to);
            id == conv_id
        })
        .collect();
    // Sort by timestamp asc
    msgs.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
    let total = msgs.len() as u64;
    let ps = if page_size == 0 { 50 } else { page_size.min(50) } as usize;
    let start = ((page.max(1) - 1) * ps as u32) as usize;
    let end = (start + ps).min(msgs.len());
    let slice = if start < end { &msgs[start..end] } else { &[] };
    let dtos = slice
        .iter()
        .map(|m| MessageDto {
            id: m.id.clone(),
            from: m.from.clone(),
            to: m.to.clone(),
            r#type: match m.channel {
                Channel::Sms => "sms".into(),
                Channel::Mms => "mms".into(),
                Channel::Email => "email".into(),
            },
            snippet: crate::snippet::make_snippet(Some(&m.body), snippet_len),
            timestamp: m.timestamp.clone(),
        })
        .collect();
    (dtos, total)
}
