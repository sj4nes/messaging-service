package state

import (
	"sync"

	"github.com/sj4nes/messaging-service/go/internal/resilience"
)

// ProviderBreakers holds per-provider circuit breakers (name -> breaker).
type ProviderBreakers struct {
	mu sync.RWMutex
	m  map[string]*resilience.Breaker
}

func NewProviderBreakers() *ProviderBreakers {
	return &ProviderBreakers{m: map[string]*resilience.Breaker{}}
}

func (p *ProviderBreakers) Insert(name string, b *resilience.Breaker) {
	p.mu.Lock()
	defer p.mu.Unlock()
	p.m[name] = b
}

func (p *ProviderBreakers) Get(name string) (*resilience.Breaker, bool) {
	p.mu.RLock()
	defer p.mu.RUnlock()
	b, ok := p.m[name]
	return b, ok
}
