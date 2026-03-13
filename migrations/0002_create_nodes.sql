CREATE TABLE nodes (
    id           TEXT PRIMARY KEY,
    user_id      UUID,
    type         TEXT NOT NULL,
    category     TEXT NOT NULL,
    title        TEXT NOT NULL,
    content      TEXT,
    metadata     JSONB NOT NULL DEFAULT '{}',
    embedding    vector(1536),
    access_count INTEGER NOT NULL DEFAULT 0,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT now()
);
