package api

import (
	"encoding/json"
	"net/http"
	"strings"

	"github.com/sj4nes/messaging-service/go/api/models"
)

func webhookSmsHandler(w http.ResponseWriter, r *http.Request) {
	if r.Header.Get("Content-Type") != "application/json" {
		writeError(w, http.StatusUnsupportedMediaType, "unsupported media type")
		return
	}
	var req models.SmsRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		writeError(w, http.StatusBadRequest, "invalid json body")
		return
	}
	typ := strings.ToLower(strings.TrimSpace(req.Type))
	if typ != "sms" && typ != "mms" {
		writeError(w, http.StatusBadRequest, "invalid type")
		return
	}
	if typ == "mms" {
		if len(req.Attachments) == 0 {
			writeError(w, http.StatusBadRequest, "mms requires at least one attachment")
			return
		}
	}
	// Persist inbound SMS/MMS webhook event
	if err := Store.CreateInboundSmsEvent(r.Context(), &req); err != nil {
		writeError(w, http.StatusInternalServerError, "failed to persist inbound event")
		return
	}
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(http.StatusAccepted)
	_ = json.NewEncoder(w).Encode(models.Accepted{Status: "accepted"})
	if reg := metricsFromContext(r); reg != nil {
		reg.IncWorkerProcessed()
	}
}

func webhookEmailHandler(w http.ResponseWriter, r *http.Request) {
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
	// Persist inbound Email webhook event
	if err := Store.CreateInboundEmailEvent(r.Context(), &req); err != nil {
		writeError(w, http.StatusInternalServerError, "failed to persist inbound event")
		return
	}
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(http.StatusAccepted)
	_ = json.NewEncoder(w).Encode(models.Accepted{Status: "accepted"})
	if reg := metricsFromContext(r); reg != nil {
		reg.IncWorkerProcessed()
	}
}
