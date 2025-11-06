use serde::{Deserialize, Serialize};

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailRequest {
    pub from: String,
    pub to: String,
    pub body: String,                      // may contain HTML
    pub attachments: Option<Vec<String>>,  // URLs
    pub timestamp: String,
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
