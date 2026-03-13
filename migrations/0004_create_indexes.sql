-- Node type/category indexes
CREATE INDEX idx_nodes_type ON nodes(type);
CREATE INDEX idx_nodes_category ON nodes(category);
CREATE INDEX idx_nodes_user ON nodes(user_id);

-- Full-text search (simple tokenizer for Phase 1; upgrade to Japanese later)
CREATE INDEX idx_nodes_fts ON nodes
    USING gin(to_tsvector('simple', coalesce(title, '') || ' ' || coalesce(content, '')));

-- Vector search (HNSW for better recall)
CREATE INDEX idx_nodes_embedding ON nodes
    USING hnsw (embedding vector_cosine_ops);

-- Inbox deduplication: (source, external_id) must be unique for inbox nodes
CREATE UNIQUE INDEX idx_inbox_dedup ON nodes ((metadata->>'source'), (metadata->>'external_id'))
    WHERE type = 'inbox';

-- Edge lookup
CREATE INDEX idx_edges_from ON edges(from_id);
CREATE INDEX idx_edges_to ON edges(to_id);
