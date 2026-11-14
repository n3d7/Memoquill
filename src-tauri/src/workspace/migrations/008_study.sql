CREATE TABLE study_sessions (
 id TEXT PRIMARY KEY,
 unit_id TEXT NOT NULL REFERENCES units(id),
 composition_id TEXT NOT NULL REFERENCES lesson_revisions(id),
 revision INTEGER NOT NULL CHECK(revision >= 0),
 data TEXT NOT NULL CHECK(json_valid(data) AND length(CAST(data AS BLOB)) <= 1048576),
 digest TEXT NOT NULL
) STRICT;
CREATE TABLE study_attempts (
 id TEXT PRIMARY KEY,
 session_id TEXT NOT NULL REFERENCES study_sessions(id),
 item_index INTEGER NOT NULL CHECK(item_index >= 0),
 data TEXT NOT NULL CHECK(json_valid(data) AND length(CAST(data AS BLOB)) <= 131072),
 digest TEXT NOT NULL,
 UNIQUE(session_id,item_index)
) STRICT;
CREATE TABLE study_judgments (
 id TEXT PRIMARY KEY,
 attempt_id TEXT NOT NULL REFERENCES study_attempts(id),
 data TEXT NOT NULL CHECK(json_valid(data) AND length(CAST(data AS BLOB)) <= 16384),
 digest TEXT NOT NULL
) STRICT;
CREATE TABLE study_operations (
 id TEXT PRIMARY KEY,
 session_id TEXT NOT NULL REFERENCES study_sessions(id),
 request_hash TEXT NOT NULL,
 result TEXT NOT NULL CHECK(json_valid(result) AND length(CAST(result AS BLOB)) <= 1048576),
 digest TEXT NOT NULL
) STRICT;
