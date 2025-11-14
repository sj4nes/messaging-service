package providers

import (
	"sync"
	"sync/atomic"
	"time"

	"github.com/sj4nes/messaging-service/go/internal/config"
)

var rngState uint64
var rngOnce sync.Once

// nextRoll returns a pseudo-random value 0..99 using a deterministic, seeded splitmix64-style update.
// We seed once when cfg is non-nil and the configured seed is non-zero; otherwise use current time.
func nextRoll(seed int64) uint32 {
	rngOnce.Do(func() {
		if seed == 0 {
			seed = time.Now().UnixNano()
		}
		atomic.StoreUint64(&rngState, uint64(seed))
	})
	// splitmix64-like update: new = old*6364136223846793005 + 1442695040888963407
	for {
		old := atomic.LoadUint64(&rngState)
		newv := old*6364136223846793005 + 1442695040888963407
		if atomic.CompareAndSwapUint64(&rngState, old, newv) {
			// downshift to reduce to 0..99
			return uint32((newv >> 1) % 100)
		}
	}
}

// PickOutcomeForProvider picks a mock provider outcome using percentages in cfg. Deterministic when a non-zero seed provided.
// Percentages (timeout, rate limit, error) are checked in that order and otherwise success is returned.
func PickOutcomeForProvider(name string, cfg *config.Config) Outcome {
	if cfg == nil {
		return OutcomeSuccess
	}
	// Build percentages; ensure values are within 0-100
	to := clampPct(cfg.ProviderTimeoutPct)
	rl := clampPct(cfg.ProviderRatelimitPct)
	er := clampPct(cfg.ProviderErrorPct)

	// select seed: provider-specific override if present
	seed := cfg.ProviderSeed
	if name == "sms-mms" && cfg.ProviderSmsSeed != 0 {
		seed = cfg.ProviderSmsSeed
	}
	if name == "email" && cfg.ProviderEmailSeed != 0 {
		seed = cfg.ProviderEmailSeed
	}

	roll := nextRoll(seed)
	if roll < uint32(to) {
		return OutcomeTimeout
	}
	if roll < uint32(to+rl) {
		return OutcomeRateLimited
	}
	if roll < uint32(to+rl+er) {
		return OutcomeError
	}
	return OutcomeSuccess
}

func clampPct(v int) int {
	if v < 0 {
		return 0
	}
	if v > 100 {
		return 100
	}
	return v
}
