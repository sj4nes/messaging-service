use serde::{Deserialize, Serialize};

use crate::config::ApiConfig;

pub trait Validate {
    fn validate(&self, api: &ApiConfig) -> Result<(), String>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmsEmailBase {
    pub from: String,
    pub to: String,
    pub body: String,
    pub attachments: Option<Vec<String>>, // URLs for MMS/email
    pub timestamp: String,                // ISO-8601
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmsRequest {
    pub from: String,
    pub to: String,
    pub r#type: String, // "sms" or "mms"
    pub body: String,
    pub attachments: Option<Vec<String>>, // present for mms
    pub timestamp: String,
}

impl Validate for SmsRequest {
    fn validate(&self, api: &ApiConfig) -> Result<(), String> {
        if self.from.trim().is_empty() || self.to.trim().is_empty() {
            return Err("'from' and 'to' are required".into());
        }
        if self.body.trim().is_empty() {
            return Err("'body' is required".into());
        }
        let t = self.r#type.to_ascii_lowercase();
        if t != "sms" && t != "mms" {
            return Err("'type' must be 'sms' or 'mms'".into());
        }
        if let Some(atts) = &self.attachments {
            if atts.len() > api.max_attachments {
                return Err(format!(
                    "too many attachments (max {})",
                    api.max_attachments
                ));
            }
            if t == "mms" && atts.is_empty() {
                return Err("mms requires at least one attachment".into());
            }
        } else if t == "mms" {
            return Err("mms requires at least one attachment".into());
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailRequest {
    pub from: String,
    pub to: String,
    pub body: String,                     // may contain HTML
    pub attachments: Option<Vec<String>>, // URLs
    pub timestamp: String,
}

impl Validate for EmailRequest {
    fn validate(&self, api: &ApiConfig) -> Result<(), String> {
        if self.from.trim().is_empty() || self.to.trim().is_empty() {
            return Err("'from' and 'to' are required".into());
        }
        if self.body.trim().is_empty() {
            return Err("'body' is required".into());
        }
        if let Some(atts) = &self.attachments {
            if atts.len() > api.max_attachments {
                return Err(format!(
                    "too many attachments (max {})",
                    api.max_attachments
                ));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookSmsRequest {
    pub from: String,
    pub to: String,
    pub r#type: String, // "sms" or "mms"
    pub messaging_provider_id: String,
    pub body: String,
    pub attachments: Option<Vec<String>>, // present for mms
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookEmailRequest {
    pub from: String,
    pub to: String,
    pub xillio_id: String,
    pub body: String,
    pub attachments: Option<Vec<String>>,
    pub timestamp: String,
}

// --------- Paging DTOs (US3/US4) ---------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageMeta {
    pub page: u32,
    pub page_size: u32,
    pub total: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListResponse<T> {
    pub items: Vec<T>,
    pub meta: PageMeta,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationDto {
    pub id: String,
    pub key: String,
    pub message_count: u32,
    pub last_activity_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageDto {
    pub id: String,
    pub from: String,
    pub to: String,
    pub r#type: String,
    pub snippet: String,
    pub timestamp: String,
}

// --------- Provider Mock Inbound (US2) ---------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "channel")]
pub enum ProviderInboundRequest {
    #[serde(rename = "sms")]
    Sms(SmsInbound),
    #[serde(rename = "mms")]
    Mms(SmsInbound),
    #[serde(rename = "email")]
    Email(EmailInbound),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmsInbound {
    pub from: String,
    pub to: String,
    pub r#type: String, // "sms" or "mms"
    pub body: String,
    pub attachments: Option<Vec<String>>, // present for mms
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailInbound {
    pub from: String,
    pub to: String,
    pub body: String,
    pub attachments: Option<Vec<String>>, // URLs
    pub timestamp: String,
}

impl Validate for SmsInbound {
    fn validate(&self, api: &ApiConfig) -> Result<(), String> {
        if self.from.trim().is_empty() || self.to.trim().is_empty() {
            return Err("'from' and 'to' are required".into());
        }
        if self.body.trim().is_empty() {
            return Err("'body' is required".into());
        }
        let t = self.r#type.to_ascii_lowercase();
        if t != "sms" && t != "mms" {
            return Err("'type' must be 'sms' or 'mms'".into());
        }
        if let Some(atts) = &self.attachments {
            if atts.len() > api.max_attachments {
                return Err(format!(
                    "too many attachments (max {})",
                    api.max_attachments
                ));
            }
            if t == "mms" && atts.is_empty() {
                return Err("mms requires at least one attachment".into());
            }
        } else if t == "mms" {
            return Err("mms requires at least one attachment".into());
        }
        Ok(())
    }
}

impl Validate for EmailInbound {
    fn validate(&self, api: &ApiConfig) -> Result<(), String> {
        if self.from.trim().is_empty() || self.to.trim().is_empty() {
            return Err("'from' and 'to' are required".into());
        }
        if self.body.trim().is_empty() {
            return Err("'body' is required".into());
        }
        if let Some(atts) = &self.attachments {
            if atts.len() > api.max_attachments {
                return Err(format!(
                    "too many attachments (max {})",
                    api.max_attachments
                ));
            }
        }
        Ok(())
    }
}
