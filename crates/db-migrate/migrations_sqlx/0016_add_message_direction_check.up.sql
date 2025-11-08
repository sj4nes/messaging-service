-- Enforce direction constraint values (UP)
ALTER TABLE messages
    ADD CONSTRAINT messages_direction_check_valid
    CHECK (direction IN ('inbound','outbound'));
