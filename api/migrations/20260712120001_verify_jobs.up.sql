-- no-transaction
CREATE TABLE IF NOT EXISTS verify_jobs(
    guild_id    NUMERIC(20, 0) NOT NULL PRIMARY KEY, -- one job row per guild, ever
    generation  BIGINT  NOT NULL DEFAULT 1,
    status      TEXT    NOT NULL,                    -- pending | running | succeeded | failed
    scope       TEXT    NOT NULL,                    -- JSON ReconScope
    cursor_user NUMERIC(20, 0),                      -- last processed user_id, NULL = start
    total       INTEGER NOT NULL DEFAULT 0,
    processed   INTEGER NOT NULL DEFAULT 0,
    errors      INTEGER NOT NULL DEFAULT 0,
    counts      TEXT    NOT NULL DEFAULT '{}',       -- JSON {role_id: matched_count} accumulator
    lease_until TIMESTAMP,
    created_at  TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at  TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
