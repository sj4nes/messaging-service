package main

import (
	"context"
	"fmt"
	"os"
	"time"

	dbmigrate "github.com/sj4nes/messaging-service/go/internal/db/migrate"
)

func main() {
	mdir := os.Getenv("MIGRATIONS_DIR")
	if mdir == "" {
		mdir = "../crates/db-migrate/migrations_sqlx"
	}

	dbURL := os.Getenv("DATABASE_URL")
	if dbURL == "" {
		fmt.Fprintln(os.Stderr, "DATABASE_URL not set; aborting")
		os.Exit(2)
	}

	ctx, cancel := context.WithTimeout(context.Background(), 60*time.Second)
	defer cancel()

	if err := dbmigrate.ApplyUp(ctx, mdir, dbURL); err != nil {
		fmt.Fprintf(os.Stderr, "migration apply failed: %v\n", err)
		os.Exit(1)
	}
	fmt.Println("migrations applied successfully")
}
