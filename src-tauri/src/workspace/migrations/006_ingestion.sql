CREATE TABLE operations_next (
    id TEXT PRIMARY KEY, workspace_id TEXT NOT NULL REFERENCES workspace_meta(id), principal TEXT NOT NULL,
    request_hash TEXT NOT NULL, kind TEXT NOT NULL CHECK(kind IN ('preferences','asset','organization','document','annotation','reading_position','source','lesson','ingestion')),
    state TEXT NOT NULL CHECK(state IN ('prepared','applied','interrupted')),
    result TEXT CHECK(result IS NULL OR json_valid(result)), error TEXT CHECK(error IS NULL OR json_valid(error))
);
INSERT INTO operations_next SELECT * FROM operations;
DROP TABLE operations;
ALTER TABLE operations_next RENAME TO operations;
CREATE TABLE ingestion_jobs (
    id TEXT PRIMARY KEY NOT NULL,
    operation_id TEXT UNIQUE NOT NULL REFERENCES operations(id) DEFERRABLE INITIALLY DEFERRED,
    request_hash TEXT NOT NULL,
    source_id TEXT NOT NULL REFERENCES original_sources(id),
    data TEXT NOT NULL CHECK(json_valid(data))
) STRICT;

CREATE VIRTUAL TABLE search_fts USING fts5(resource_id UNINDEXED, title, body, tags, tokenize='trigram');
CREATE TABLE search_index_state (id INTEGER PRIMARY KEY CHECK(id=1), dirty INTEGER NOT NULL CHECK(dirty IN (0,1))) STRICT;
INSERT INTO search_index_state VALUES(1,1);
CREATE TRIGGER search_resource_insert AFTER INSERT ON resources BEGIN UPDATE search_index_state SET dirty=1 WHERE id=1; END;
CREATE TRIGGER search_resource_update AFTER UPDATE ON resources BEGIN UPDATE search_index_state SET dirty=1 WHERE id=1; END;
CREATE TRIGGER search_resource_delete AFTER DELETE ON resources BEGIN UPDATE search_index_state SET dirty=1 WHERE id=1; END;
CREATE TRIGGER search_head_insert AFTER INSERT ON resource_heads BEGIN UPDATE search_index_state SET dirty=1 WHERE id=1; END;
CREATE TRIGGER search_head_update AFTER UPDATE ON resource_heads BEGIN UPDATE search_index_state SET dirty=1 WHERE id=1; END;
CREATE TRIGGER search_head_delete AFTER DELETE ON resource_heads BEGIN UPDATE search_index_state SET dirty=1 WHERE id=1; END;
CREATE TRIGGER search_revision_update AFTER UPDATE ON resource_revisions BEGIN UPDATE search_index_state SET dirty=1 WHERE id=1; END;
CREATE TRIGGER search_revision_delete AFTER DELETE ON resource_revisions BEGIN UPDATE search_index_state SET dirty=1 WHERE id=1; END;
CREATE TRIGGER search_tag_insert AFTER INSERT ON tags BEGIN UPDATE search_index_state SET dirty=1 WHERE id=1; END;
CREATE TRIGGER search_tag_update AFTER UPDATE ON tags BEGIN UPDATE search_index_state SET dirty=1 WHERE id=1; END;
CREATE TRIGGER search_tag_delete AFTER DELETE ON tags BEGIN UPDATE search_index_state SET dirty=1 WHERE id=1; END;
CREATE TRIGGER search_tag_resource_insert AFTER INSERT ON tag_resources BEGIN UPDATE search_index_state SET dirty=1 WHERE id=1; END;
CREATE TRIGGER search_tag_resource_update AFTER UPDATE ON tag_resources BEGIN UPDATE search_index_state SET dirty=1 WHERE id=1; END;
CREATE TRIGGER search_tag_resource_delete AFTER DELETE ON tag_resources BEGIN UPDATE search_index_state SET dirty=1 WHERE id=1; END;
