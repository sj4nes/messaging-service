package migrate

import (
	"context"
	"errors"
	"fmt"
	"net/url"
	"os"
	"os/exec"
	"time"
)

// ApplyUp runs `migrate` CLI to apply up migrations at the given path against the given DB URL.
// It is a best-effort helper for dev/CI; in minimal containers the CLI may be absent.
func ApplyUp(ctx context.Context, migrationsPath, databaseURL string) error {
	if migrationsPath == "" || databaseURL == "" {
		return errors.New("migrations path and database url are required")
	}
	// Some postgres drivers (used by the migrate CLI) will require an explicit
	// sslmode. For local Docker/Postgres use cases we default sslmode to
	// "disable" when it is not already provided on the connection string.
	// This mirrors the behavior the Rust db-migrate (sqlx) helper has when
	// connecting to local instances and avoids the "SSL is not enabled on
	// the server" error that occurs when the driver expects SSL by default.
	if u, err := url.Parse(databaseURL); err == nil {
		// Only set sslmode for postgres scheme and when not present already.
		if u.Scheme == "postgres" || u.Scheme == "postgresql" {
			q := u.Query()
			if q.Get("sslmode") == "" {
				q.Set("sslmode", "disable")
				u.RawQuery = q.Encode()
				databaseURL = u.String()
			}
		}
	}
	_, err := exec.LookPath("migrate")
	if err != nil {
		return fmt.Errorf("migrate CLI not found: %w", err)
	}
	cctx, cancel := context.WithTimeout(ctx, 60*time.Second)
	defer cancel()
	cmd := exec.CommandContext(cctx, "migrate", "-path", migrationsPath, "-database", databaseURL, "up")
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr
	if err := cmd.Run(); err != nil {
		return fmt.Errorf("migration apply failed: %w", err)
	}
	return nil
}
