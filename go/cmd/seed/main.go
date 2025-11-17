package main

import (
"context"
"fmt"
"os"
"time"

"github.com/jackc/pgx/v5/pgxpool"
"github.com/sj4nes/messaging-service/go/internal/db/seed"
)

func main() {
	dbURL := os.Getenv("DATABASE_URL")
	if dbURL == "" {
		fmt.Fprintln(os.Stderr, "DATABASE_URL not set")
		os.Exit(1)
	}

	pool, err := pgxpool.New(context.Background(), dbURL)
	if err != nil {
		fmt.Fprintf(os.Stderr, "Failed to connect to database: %v\n", err)
		os.Exit(1)
	}
	defer pool.Close()

	ctx, cancel := context.WithTimeout(context.Background(), 30*time.Second)
	defer cancel()

	seed.SeedMinimumIfNeeded(ctx, pool)
	fmt.Println("Seed completed successfully")
}
