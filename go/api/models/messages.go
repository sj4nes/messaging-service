package models

// SmsRequest matches contracts in specs/006-unified-messaging
type SmsRequest struct {
    From        string   `json:"from"`
    To          string   `json:"to"`
    Type        string   `json:"type"` // sms|mms
    Body        string   `json:"body"`
    Attachments []string `json:"attachments,omitempty"`
    Timestamp   string   `json:"timestamp"`
}

// EmailRequest matches contracts in specs/006-unified-messaging
type EmailRequest struct {
    From        string   `json:"from"`
    To          string   `json:"to"`
    Body        string   `json:"body"`
    Attachments []string `json:"attachments,omitempty"`
    Timestamp   string   `json:"timestamp"`
}

type Accepted struct {
    Status string `json:"status"` // "accepted"
}
