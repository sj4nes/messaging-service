package secrets

import (
	"errors"
	"os"
)

// Dev is a development provider pulling from environment variables.
// Returns error if key not present.
type Dev struct{}

func (d Dev) Get(key string) (string, error) {
	v := os.Getenv(key)
	if v == "" {
		return "", errors.New("secret not found")
	}
	return v, nil
}
