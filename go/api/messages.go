package api

import (
	"encoding/json"
	"net/http"

	"github.com/sj4nes/messaging-service/go/api/models"
)

func smsHandler(w http.ResponseWriter, r *http.Request) {
	if r.Header.Get("Content-Type") != "application/json" {
		writeError(w, http.StatusUnsupportedMediaType, "unsupported media type")
		return
	}
	var req models.SmsRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		writeError(w, http.StatusBadRequest, "invalid json body")
		return
	}
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(http.StatusAccepted)
	_ = json.NewEncoder(w).Encode(models.Accepted{Status: "accepted"})
}

func emailHandler(w http.ResponseWriter, r *http.Request) {
	if r.Header.Get("Content-Type") != "application/json" {
		writeError(w, http.StatusUnsupportedMediaType, "unsupported media type")
		return
	}
	var req models.EmailRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		writeError(w, http.StatusBadRequest, "invalid json body")
		return
	}
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(http.StatusAccepted)
	_ = json.NewEncoder(w).Encode(models.Accepted{Status: "accepted"})
}

// Routes registers message endpoints.
func Routes(mux interface {
	Post(string, http.HandlerFunc)
	Get(string, http.HandlerFunc)
}) {
	mux.Post("/api/messages/sms", smsHandler)
	mux.Post("/api/messages/email", emailHandler)
	// Conversations
	ConversationRoutes(mux)
}
