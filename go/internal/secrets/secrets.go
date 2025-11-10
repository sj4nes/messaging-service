package secrets

// Provider defines secret retrieval behavior.
type Provider interface {
    Get(key string) (string, error)
}
