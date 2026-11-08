CREATE TABLE source_revisions (
    resource_id TEXT PRIMARY KEY NOT NULL REFERENCES resources(id),
    revision INTEGER NOT NULL CHECK(revision >= 0 AND revision <= 9007199254740991)
) STRICT;

CREATE TABLE original_sources (
    id TEXT PRIMARY KEY NOT NULL,
    resource_id TEXT NOT NULL REFERENCES resources(id),
    digest TEXT NOT NULL REFERENCES assets(digest),
    filename TEXT NOT NULL,
    media_type TEXT NOT NULL,
    attribution TEXT,
    license TEXT,
    origin_url TEXT,
    revision INTEGER NOT NULL CHECK(revision > 0 AND revision <= 9007199254740991),
    byte_length INTEGER NOT NULL CHECK(byte_length >= 0 AND byte_length <= 67108864),
    UNIQUE(resource_id, revision)
) STRICT;
