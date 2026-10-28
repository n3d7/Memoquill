CREATE TABLE operations (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspace_meta(id),
    principal TEXT NOT NULL,
    request_hash TEXT NOT NULL,
    kind TEXT NOT NULL CHECK (kind IN ('preferences', 'asset')),
    state TEXT NOT NULL CHECK (state IN ('prepared', 'applied', 'interrupted')),
    result TEXT CHECK (result IS NULL OR json_valid(result)),
    error TEXT CHECK (error IS NULL OR json_valid(error))
);
CREATE TABLE assets (
    digest TEXT PRIMARY KEY CHECK (length(digest) = 64),
    byte_length INTEGER NOT NULL CHECK (byte_length >= 0),
    registered_by TEXT NOT NULL REFERENCES operations(id)
);
