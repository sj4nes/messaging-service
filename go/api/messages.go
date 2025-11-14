package api

import (
	"encoding/json"
	"net/http"
	"strings"

	"github.com/sj4nes/messaging-service/go/api/models"
	"github.com/sj4nes/messaging-service/go/internal/metrics"
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
	// Basic validations for parity with Rust implementation expectations
	typ := strings.ToLower(strings.TrimSpace(req.Type))
	if typ != "sms" && typ != "mms" {
		writeError(w, http.StatusBadRequest, "invalid type")
		return
	}
	if typ == "mms" {
		if len(req.Attachments) == 0 { // nil or empty
			writeError(w, http.StatusBadRequest, "mms requires at least one attachment")
			return
		}
	}
	// Enqueue the outbound SMS/MMS via configured store.
	if reg := metricsFromContext(r); reg != nil {
		reg.IncEnqueueAttempt()
	}
	if err := Store.CreateSmsMessage(r.Context(), &req); err != nil {
		if reg := metricsFromContext(r); reg != nil {
			reg.IncEnqueueFailure()
		}
		writeError(w, http.StatusInternalServerError, "failed to enqueue message")
		return
	}
	if reg := metricsFromContext(r); reg != nil {
		reg.IncEnqueueSuccess()
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
	if strings.TrimSpace(req.Body) == "" {
		writeError(w, http.StatusBadRequest, "empty body")
		return
	}
	if reg := metricsFromContext(r); reg != nil {
		reg.IncEnqueueAttempt()
	}
	if err := Store.CreateEmailMessage(r.Context(), &req); err != nil {
		if reg := metricsFromContext(r); reg != nil {
			reg.IncEnqueueFailure()
		}
		writeError(w, http.StatusInternalServerError, "failed to enqueue message")
		return
	}
	if reg := metricsFromContext(r); reg != nil {
		reg.IncEnqueueSuccess()
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
	// Webhooks (inbound events)
	mux.Post("/api/webhooks/sms", webhookSmsHandler)
	mux.Post("/api/webhooks/email", webhookEmailHandler)
	// Conversations
	ConversationRoutes(mux)
}

// metricsFromContext extracts *metrics.Registry if previously set by middleware.
// For now we use a lightweight approach: tests will still pass even if nil.
func metricsFromContext(r *http.Request) *metrics.Registry {
	if v := r.Context().Value(metricsContextKey{}); v != nil {
		if reg, ok := v.(*metrics.Registry); ok {
			return reg
		}
	}
	return nil
}

type metricsContextKey struct{}
