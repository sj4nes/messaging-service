package db

import "context"

// TxFunc is a function executed within a transaction boundary.
type TxFunc func(ctx context.Context) error

// TxManager abstracts transaction handling for repositories.
type TxManager interface {
	WithTx(ctx context.Context, fn TxFunc) error
}

// NoopTxManager implements TxManager without starting a real transaction.
// Useful for tests or when a connection is unavailable.
type NoopTxManager struct{}

func (NoopTxManager) WithTx(ctx context.Context, fn TxFunc) error {
	return fn(ctx)
}
