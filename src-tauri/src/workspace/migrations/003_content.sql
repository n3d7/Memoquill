CREATE TABLE operations_next (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspace_meta(id),
    principal TEXT NOT NULL,
    request_hash TEXT NOT NULL,
    kind TEXT NOT NULL CHECK (kind IN ('preferences', 'asset', 'organization', 'document', 'annotation', 'reading_position')),
    state TEXT NOT NULL CHECK (state IN ('prepared', 'applied', 'interrupted')),
    result TEXT CHECK (result IS NULL OR json_valid(result)),
    error TEXT CHECK (error IS NULL OR json_valid(error))
);
INSERT INTO operations_next SELECT * FROM operations;
CREATE TABLE assets_next (
    digest TEXT PRIMARY KEY CHECK (length(digest) = 64),
    byte_length INTEGER NOT NULL CHECK (byte_length >= 0),
    registered_by TEXT NOT NULL REFERENCES operations_next(id)
);
INSERT INTO assets_next SELECT * FROM assets;
DROP TABLE assets;
DROP TABLE operations;
ALTER TABLE operations_next RENAME TO operations;
ALTER TABLE assets_next RENAME TO assets;

CREATE TABLE library_meta (workspace_id TEXT PRIMARY KEY REFERENCES workspace_meta(id), revision INTEGER NOT NULL CHECK(revision >= 0));
CREATE TABLE subjects (id TEXT PRIMARY KEY, title TEXT NOT NULL, prior_knowledge TEXT, archived INTEGER NOT NULL CHECK(archived IN (0,1)));
CREATE TABLE goals (id TEXT PRIMARY KEY, subject_id TEXT NOT NULL REFERENCES subjects(id), title TEXT NOT NULL, criteria TEXT NOT NULL, archived INTEGER NOT NULL CHECK(archived IN (0,1)));
CREATE TABLE collections (id TEXT PRIMARY KEY, subject_id TEXT NOT NULL REFERENCES subjects(id), parent_id TEXT REFERENCES collections(id) DEFERRABLE INITIALLY DEFERRED, title TEXT NOT NULL, position INTEGER NOT NULL CHECK(position >= 0), archived INTEGER NOT NULL CHECK(archived IN (0,1)));
CREATE TABLE resources (id TEXT PRIMARY KEY, title TEXT NOT NULL, archived INTEGER NOT NULL CHECK(archived IN (0,1)));
CREATE TABLE units (id TEXT PRIMARY KEY, title TEXT NOT NULL, objective TEXT NOT NULL, archived INTEGER NOT NULL CHECK(archived IN (0,1)));
CREATE TABLE unit_goals (unit_id TEXT NOT NULL REFERENCES units(id), goal_id TEXT NOT NULL REFERENCES goals(id), position INTEGER NOT NULL, PRIMARY KEY(unit_id,goal_id));
CREATE TABLE unit_resources (unit_id TEXT NOT NULL REFERENCES units(id), resource_id TEXT NOT NULL REFERENCES resources(id), position INTEGER NOT NULL, PRIMARY KEY(unit_id,resource_id));
CREATE TABLE placements (id TEXT PRIMARY KEY, collection_id TEXT NOT NULL REFERENCES collections(id), resource_id TEXT REFERENCES resources(id), unit_id TEXT REFERENCES units(id), position INTEGER NOT NULL CHECK(position >= 0), archived INTEGER NOT NULL CHECK(archived IN (0,1)), CHECK((resource_id IS NULL) != (unit_id IS NULL)));
CREATE TABLE paths (id TEXT PRIMARY KEY, title TEXT NOT NULL, archived INTEGER NOT NULL CHECK(archived IN (0,1)));
CREATE TABLE path_steps (id TEXT PRIMARY KEY, path_id TEXT NOT NULL REFERENCES paths(id), unit_id TEXT NOT NULL REFERENCES units(id), position INTEGER NOT NULL CHECK(position >= 0), UNIQUE(path_id,position));
CREATE TABLE relations (id TEXT PRIMARY KEY, kind TEXT NOT NULL CHECK(kind IN ('prerequisite','related','reference')), from_unit TEXT NOT NULL REFERENCES units(id), to_unit TEXT NOT NULL REFERENCES units(id));
CREATE TABLE tags (id TEXT PRIMARY KEY, title TEXT NOT NULL);
CREATE TABLE tag_resources (tag_id TEXT NOT NULL REFERENCES tags(id), resource_id TEXT NOT NULL REFERENCES resources(id), position INTEGER NOT NULL, PRIMARY KEY(tag_id,resource_id));
CREATE TABLE resource_revisions (id TEXT PRIMARY KEY, resource_id TEXT NOT NULL REFERENCES resources(id), revision INTEGER NOT NULL CHECK(revision > 0), document TEXT NOT NULL CHECK(json_valid(document)), digest TEXT NOT NULL, UNIQUE(resource_id,revision));
CREATE TABLE resource_heads (resource_id TEXT PRIMARY KEY REFERENCES resources(id), revision_id TEXT NOT NULL UNIQUE REFERENCES resource_revisions(id));
CREATE TABLE annotations (id TEXT PRIMARY KEY, resource_id TEXT NOT NULL REFERENCES resources(id), revision_id TEXT NOT NULL REFERENCES resource_revisions(id), revision INTEGER NOT NULL CHECK(revision > 0), anchor TEXT NOT NULL CHECK(json_valid(anchor)), body TEXT NOT NULL);
CREATE TABLE reading_positions (resource_id TEXT PRIMARY KEY REFERENCES resources(id), revision_id TEXT NOT NULL REFERENCES resource_revisions(id), block_id TEXT NOT NULL, offset_utf16 INTEGER NOT NULL CHECK(offset_utf16 >= 0), revision INTEGER NOT NULL CHECK(revision > 0));
CREATE INDEX placements_resource ON placements(resource_id);
CREATE INDEX collections_parent ON collections(subject_id,parent_id,position);
CREATE INDEX relations_from ON relations(kind,from_unit);
