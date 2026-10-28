CREATE TABLE schema_migrations (
    version INTEGER PRIMARY KEY,
    checksum TEXT NOT NULL
);
CREATE TABLE workspace_meta (
    singleton INTEGER PRIMARY KEY CHECK (singleton = 1),
    id TEXT NOT NULL UNIQUE
);
CREATE TABLE preferences (
    workspace_id TEXT PRIMARY KEY REFERENCES workspace_meta(id),
    revision INTEGER NOT NULL CHECK (revision >= 0),
    data TEXT NOT NULL CHECK (json_valid(data))
);
