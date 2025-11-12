-- name: ListConversations :many
SELECT id::text, key, channel, participant_a, participant_b, message_count, last_activity_at
FROM conversations
ORDER BY last_activity_at DESC
LIMIT $1 OFFSET $2;
