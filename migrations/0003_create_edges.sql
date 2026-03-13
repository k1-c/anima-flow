CREATE TABLE edges (
    from_id   TEXT NOT NULL REFERENCES nodes(id) ON DELETE CASCADE,
    to_id     TEXT NOT NULL REFERENCES nodes(id) ON DELETE CASCADE,
    relation  TEXT NOT NULL,
    weight    REAL NOT NULL DEFAULT 1.0,
    context   TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (from_id, to_id)
);
