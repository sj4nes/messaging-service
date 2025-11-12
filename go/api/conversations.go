package api

import (
	"encoding/json"
	"net/http"
	"strconv"
	"strings"

	"github.com/go-chi/chi/v5"
	"github.com/sj4nes/messaging-service/go/api/models"
)

func listConversationsHandler(w http.ResponseWriter, r *http.Request) {
	// Enforce JSON Accept header for parity; allow default (no Accept) to proceed
	if acc := r.Header.Get("Accept"); acc != "" {
		if !acceptsJSON(acc) {
			w.WriteHeader(http.StatusNotAcceptable)
			return
		}
	}
	page, size := parsePaging(r, 1, 50)
	items, total, _ := Store.ListConversations(r.Context(), page, size)
	resp := models.ListResponse[models.ConversationDto]{
		Items: items,
		Meta:  models.PageMeta{Page: uint32(page), PageSize: uint32(size), Total: total},
	}
	w.Header().Set("Content-Type", "application/json")
	_ = json.NewEncoder(w).Encode(resp)
}

func listConversationMessagesHandler(w http.ResponseWriter, r *http.Request) {
	if acc := r.Header.Get("Accept"); acc != "" {
		if !acceptsJSON(acc) {
			w.WriteHeader(http.StatusNotAcceptable)
			return
		}
	}
	id := chi.URLParam(r, "id")
	page, size := parsePaging(r, 1, 50)
	items, total, _ := Store.ListMessages(r.Context(), id, page, size)
	resp := models.ListResponse[models.MessageDto]{
		Items: items,
		Meta:  models.PageMeta{Page: uint32(page), PageSize: uint32(size), Total: total},
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

// acceptsJSON performs a minimal check for application/json or wildcards
func acceptsJSON(accept string) bool {
	a := strings.ToLower(accept)
	return strings.Contains(a, "application/json") || strings.Contains(a, "*/*") || strings.Contains(a, "application/*")
}

// ConversationRoutes registers conversation list endpoints.
func ConversationRoutes(mux interface {
	Get(string, http.HandlerFunc)
}) {
	mux.Get("/api/conversations", listConversationsHandler)
	mux.Get("/api/conversations/{id}/messages", listConversationMessagesHandler)
}
