-- Migration 001: Initial schema

CREATE TABLE IF NOT EXISTS links (
    id TEXT PRIMARY KEY NOT NULL,
    short_code TEXT NOT NULL UNIQUE,
    target_url TEXT NOT NULL,
    created_at TEXT NOT NULL,
    expires_at TEXT
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_links_short_code ON links(short_code);
CREATE INDEX IF NOT EXISTS idx_links_expires_at ON links(expires_at);
