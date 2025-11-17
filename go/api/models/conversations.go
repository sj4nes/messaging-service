package models

type ConversationDto struct {
	ID           string `json:"id"`
	Key          string `json:"key"`
	Channel      string `json:"channel,omitempty"`
	ParticipantA string `json:"participant_a,omitempty"`
	ParticipantB string `json:"participant_b,omitempty"`
	MessageCount uint32 `json:"message_count"`
	LastActivity string `json:"last_activity_at"`
}

type MessageDto struct {
	ID        string `json:"id"`
	Direction string `json:"direction,omitempty"` // outbound|inbound (older contracts use type)
	Channel   string `json:"channel,omitempty"`
	From      string `json:"from"`
	To        string `json:"to"`
	Body      string `json:"body,omitempty"`
	Snippet   string `json:"snippet,omitempty"`
	Timestamp string `json:"timestamp"`
}

type PageMeta struct {
	Page     uint32 `json:"page"`
	PageSize uint32 `json:"page_size"`
	Total    uint64 `json:"total"`
}

type ListResponse[T any] struct {
	Items []T      `json:"items"`
	Meta  PageMeta `json:"meta"`
}
