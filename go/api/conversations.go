package api

import (
    "encoding/json"
    "net/http"
    "strconv"

    "github.com/go-chi/chi/v5"
    "github.com/sj4nes/messaging-service/go/api/models"
)

// In-memory placeholder data until persistence layer (US4) provides real repository.
var sampleConversations = []models.ConversationDto{
    {ID: "c1", Key: "conv:c1", Channel: "sms", ParticipantA: "+15550001", ParticipantB: "+15550002", MessageCount: 2, LastActivity: "2025-11-10T12:00:00Z"},
    {ID: "c2", Key: "conv:c2", Channel: "email", ParticipantA: "alice@example.com", ParticipantB: "bob@example.com", MessageCount: 1, LastActivity: "2025-11-10T12:05:00Z"},
}

var sampleMessages = map[string][]models.MessageDto{
    "c1": {
        {ID: "m1", Direction: "outbound", Channel: "sms", From: "+15550001", To: "+15550002", Body: "Hello there", Snippet: "Hello there", Timestamp: "2025-11-10T12:00:00Z"},
        {ID: "m2", Direction: "inbound", Channel: "sms", From: "+15550002", To: "+15550001", Body: "Hi!", Snippet: "Hi!", Timestamp: "2025-11-10T12:01:00Z"},
    },
    "c2": {
        {ID: "m3", Direction: "outbound", Channel: "email", From: "alice@example.com", To: "bob@example.com", Body: "Status update", Snippet: "Status update", Timestamp: "2025-11-10T12:05:00Z"},
    },
}

func listConversationsHandler(w http.ResponseWriter, r *http.Request) {
    page, size := parsePaging(r, 1, 50)
    // Simple slice paging.
    total := uint64(len(sampleConversations))
    start := int((page - 1) * size)
    if start > len(sampleConversations) {
        start = len(sampleConversations)
    }
    end := start + int(size)
    if end > len(sampleConversations) {
        end = len(sampleConversations)
    }
    items := sampleConversations[start:end]
    resp := models.ListResponse[models.ConversationDto]{
        Items: items,
        Meta: models.PageMeta{Page: uint32(page), PageSize: uint32(size), Total: total},
    }
    w.Header().Set("Content-Type", "application/json")
    _ = json.NewEncoder(w).Encode(resp)
}

func listConversationMessagesHandler(w http.ResponseWriter, r *http.Request) {
    id := chi.URLParam(r, "id")
    msgs := sampleMessages[id]
    page, size := parsePaging(r, 1, 50)
    total := uint64(len(msgs))
    start := int((page - 1) * size)
    if start > len(msgs) {
        start = len(msgs)
    }
    end := start + int(size)
    if end > len(msgs) {
        end = len(msgs)
    }
    items := msgs[start:end]
    resp := models.ListResponse[models.MessageDto]{
        Items: items,
        Meta: models.PageMeta{Page: uint32(page), PageSize: uint32(size), Total: total},
    }
    w.Header().Set("Content-Type", "application/json")
    _ = json.NewEncoder(w).Encode(resp)
}

func parsePaging(r *http.Request, defaultPage, defaultSize int) (int, int) {
    q := r.URL.Query()
    page := defaultPage
    size := defaultSize
    if v := q.Get("page"); v != "" {
        if n, err := strconv.Atoi(v); err == nil && n > 0 {
            page = n
        }
    }
    if v := q.Get("page_size"); v != "" {
        if n, err := strconv.Atoi(v); err == nil && n > 0 && n <= 250 {
            size = n
        }
    }
    return page, size
}

// ConversationRoutes registers conversation list endpoints.
func ConversationRoutes(mux interface {
    Get(string, http.HandlerFunc)
}) {
    mux.Get("/api/conversations", listConversationsHandler)
    mux.Get("/api/conversations/{id}/messages", listConversationMessagesHandler)
}
