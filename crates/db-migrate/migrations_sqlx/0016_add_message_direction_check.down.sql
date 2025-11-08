-- Remove direction constraint (DOWN)
ALTER TABLE messages
    DROP CONSTRAINT IF EXISTS messages_direction_check_valid;
