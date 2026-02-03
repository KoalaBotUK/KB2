-- no-transaction
CREATE TABLE audit(
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event VARCHAR,
    user_id NUMERIC(20, 0) NOT NULL, -- u64
    guild_id NUMERIC(20, 0), -- u64
    old_data VARCHAR,
    new_data VARCHAR,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);