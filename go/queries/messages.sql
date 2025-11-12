-- name: ListMessagesForConversation :many
SELECT m.id::text,
       c.channel,
       c.participant_a AS from_participant,
       c.participant_b AS to_participant,
       m.direction,
       m.sent_at AS timestamp
FROM messages m
JOIN conversations c ON c.id = m.conversation_id
WHERE c.id = $1
ORDER BY m.sent_at ASC
LIMIT $2 OFFSET $3;
