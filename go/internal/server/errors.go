package server

import (
	"errors"
	"net/http"
)

// Sentinel errors for mapping; expand as needed for parity.
var (
	ErrUnsupportedMediaType = errors.New("unsupported media type")
	ErrInvalidJSON          = errors.New("invalid json body")
	ErrRateLimited          = errors.New("rate limited")
)

// MapToStatus translates internal errors to HTTP status codes.
func MapToStatus(err error) int {
	switch {
	case errors.Is(err, ErrUnsupportedMediaType):
		return http.StatusUnsupportedMediaType
	case errors.Is(err, ErrInvalidJSON):
		return http.StatusBadRequest
	case errors.Is(err, ErrRateLimited):
		return http.StatusTooManyRequests
	default:
		return http.StatusBadRequest
	}
}
