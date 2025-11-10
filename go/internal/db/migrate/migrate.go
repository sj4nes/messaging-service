package migrate

import (
	"context"
	"errors"
	"fmt"
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
