package httpclient

import (
	"errors"
	"net/http"
	"time"

	"github.com/sj4nes/messaging-service/go/internal/security"
)

// Client wraps http.Client adding SSRF validation before outbound requests.
type Client struct {
	http      *http.Client
	validator *security.Validator
}

// Config for the HTTP client.
type Config struct {
	Timeout       time.Duration
	SSRFAllowlist []string
}

func New(cfg Config) *Client {
	v := security.NewValidator(cfg.SSRFAllowlist)
	return &Client{http: &http.Client{Timeout: cfg.Timeout}, validator: v}
}

// Do validates the URL in the request (Host) before executing.
func (c *Client) Do(req *http.Request) (*http.Response, error) {
	if req == nil || req.URL == nil {
		return nil, errors.New("nil request")
	}
	// Validate full URL string
	if err := c.validator.IsAllowed(req.URL.String()); err != nil {
		return nil, err
	}
	return c.http.Do(req)
}

// Get is a convenience method performing a GET request after validation.
func (c *Client) Get(url string) (*http.Response, error) {
	if err := c.validator.IsAllowed(url); err != nil {
		return nil, err
	}
	req, err := http.NewRequest(http.MethodGet, url, nil)
	if err != nil {
		return nil, err
	}
	return c.http.Do(req)
}
