use std::sync::{OnceLock, RwLock};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::types::{EmailInbound, ProviderInboundRequest, SmsInbound};
use super::conversations;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Direction {
	Outbound,
	Inbound,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Channel {
	Sms,
	Mms,
	Email,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredMessage {
	pub id: String,
	pub direction: Direction,
	pub channel: Channel,
	pub from: String,
	pub to: String,
	pub body: String,
	pub attachments: Option<Vec<String>>,
	pub timestamp: String,
}

fn store() -> &'static RwLock<Vec<StoredMessage>> {
	static CELL: OnceLock<RwLock<Vec<StoredMessage>>> = OnceLock::new();
	CELL.get_or_init(|| RwLock::new(Vec::new()))
}

pub fn insert_inbound(req: &ProviderInboundRequest) -> String {
	let (channel, from, to, body, attachments, timestamp) = match req {
		ProviderInboundRequest::Sms(SmsInbound { from, to, r#type, body, attachments, timestamp }) => {
			let ch = if r#type.to_ascii_lowercase() == "mms" { Channel::Mms } else { Channel::Sms };
			(ch, from.clone(), to.clone(), body.clone(), attachments.clone(), timestamp.clone())
		}
		ProviderInboundRequest::Mms(SmsInbound { from, to, body, attachments, timestamp, .. }) => {
			(Channel::Mms, from.clone(), to.clone(), body.clone(), attachments.clone(), timestamp.clone())
		}
		ProviderInboundRequest::Email(EmailInbound { from, to, body, attachments, timestamp }) => {
			(Channel::Email, from.clone(), to.clone(), body.clone(), attachments.clone(), timestamp.clone())
		}
	};
	let id = Uuid::new_v4().to_string();
	let msg = StoredMessage {
		id: id.clone(),
		direction: Direction::Inbound,
		channel,
		from,
		to,
		body,
		attachments,
		timestamp,
	};
	let lock = store();
	let mut w = lock.write().unwrap();
	w.push(msg);
	// Update conversation index
	if let Some(last) = w.last() {
		conversations::on_message_stored(last);
	}
	id
}

pub fn insert_outbound_sms(from: &str, to: &str, body: &str, attachments: &Option<Vec<String>>, timestamp: &str) -> String {
	insert_outbound(Channel::Sms, from, to, body, attachments, timestamp)
}

pub fn insert_outbound_mms(from: &str, to: &str, body: &str, attachments: &Option<Vec<String>>, timestamp: &str) -> String {
	insert_outbound(Channel::Mms, from, to, body, attachments, timestamp)
}

pub fn insert_outbound_email(from: &str, to: &str, body: &str, attachments: &Option<Vec<String>>, timestamp: &str) -> String {
	insert_outbound(Channel::Email, from, to, body, attachments, timestamp)
}

fn insert_outbound(channel: Channel, from: &str, to: &str, body: &str, attachments: &Option<Vec<String>>, timestamp: &str) -> String {
	let id = Uuid::new_v4().to_string();
	let msg = StoredMessage {
		id: id.clone(),
		direction: Direction::Outbound,
		channel,
		from: from.to_string(),
		to: to.to_string(),
		body: body.to_string(),
		attachments: attachments.clone(),
		timestamp: timestamp.to_string(),
	};
	let lock = store();
	let mut w = lock.write().unwrap();
	w.push(msg);
	if let Some(last) = w.last() {
		conversations::on_message_stored(last);
	}
	id
}

#[allow(dead_code)]
pub fn all() -> Vec<StoredMessage> {
	store().read().unwrap().clone()
}
