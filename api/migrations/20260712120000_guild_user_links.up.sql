-- no-transaction
CREATE TABLE IF NOT EXISTS guild_user_links(
    guild_id   NUMERIC(20, 0) NOT NULL, -- u64
    user_id    NUMERIC(20, 0) NOT NULL, -- u64
    links      TEXT NOT NULL,           -- JSON Vec<Link>, same shape as users.links
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (guild_id, user_id)     -- also serves keyset pagination
);
