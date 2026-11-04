CREATE TABLE operations_next (
 id TEXT PRIMARY KEY, workspace_id TEXT NOT NULL REFERENCES workspace_meta(id), principal TEXT NOT NULL,
 request_hash TEXT NOT NULL, kind TEXT NOT NULL CHECK(kind IN ('preferences','asset','organization','document','annotation','reading_position','source','lesson')),
 state TEXT NOT NULL CHECK(state IN ('prepared','applied','interrupted')),
 result TEXT CHECK(result IS NULL OR json_valid(result)), error TEXT CHECK(error IS NULL OR json_valid(error))
);
INSERT INTO operations_next SELECT * FROM operations;
DROP TABLE operations;
ALTER TABLE operations_next RENAME TO operations;
CREATE TABLE lesson_drafts (id TEXT PRIMARY KEY, unit_id TEXT NOT NULL REFERENCES units(id), actor TEXT NOT NULL, data TEXT NOT NULL CHECK(json_valid(data) AND length(CAST(data AS BLOB))<=2097152)) STRICT;
CREATE INDEX lesson_drafts_scope ON lesson_drafts(unit_id,actor);
CREATE TABLE lesson_revisions (id TEXT PRIMARY KEY, unit_id TEXT NOT NULL REFERENCES units(id), parent_id TEXT REFERENCES lesson_revisions(id), data TEXT NOT NULL CHECK(json_valid(data) AND length(CAST(data AS BLOB))<=1048576), digest TEXT NOT NULL) STRICT;
CREATE TABLE lesson_heads (unit_id TEXT PRIMARY KEY REFERENCES units(id), revision_id TEXT NOT NULL UNIQUE REFERENCES lesson_revisions(id)) STRICT;
CREATE TABLE lesson_activity_revisions (id TEXT PRIMARY KEY, activity_id TEXT NOT NULL, unit_id TEXT NOT NULL REFERENCES units(id), definition TEXT NOT NULL CHECK(json_valid(definition)), digest TEXT NOT NULL) STRICT;
CREATE TABLE lesson_activity_heads (activity_id TEXT PRIMARY KEY, unit_id TEXT NOT NULL REFERENCES units(id), revision_id TEXT NOT NULL REFERENCES lesson_activity_revisions(id)) STRICT;
CREATE TABLE lesson_resource_owners (resource_id TEXT PRIMARY KEY REFERENCES resources(id), unit_id TEXT NOT NULL REFERENCES units(id)) STRICT;
CREATE TABLE lesson_dependencies (composition_id TEXT NOT NULL REFERENCES lesson_revisions(id), kind TEXT NOT NULL CHECK(kind IN ('resource','source','activity')), target_id TEXT NOT NULL, revision_id TEXT NOT NULL, PRIMARY KEY(composition_id,kind,target_id,revision_id)) STRICT;
CREATE TABLE lesson_source_bindings (id TEXT PRIMARY KEY, unit_id TEXT NOT NULL REFERENCES units(id), source_id TEXT NOT NULL REFERENCES original_sources(id), data TEXT NOT NULL CHECK(json_valid(data)), digest TEXT NOT NULL) STRICT;
