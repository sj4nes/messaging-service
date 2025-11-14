-- name: GetProviderByName :one
SELECT id FROM providers WHERE name = $1 LIMIT 1;
