CREATE TABLE search_projection (
    resource_id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    body TEXT NOT NULL,
    tags TEXT NOT NULL
) STRICT;
CREATE TRIGGER search_projection_insert AFTER INSERT ON search_projection BEGIN UPDATE search_index_state SET dirty=1 WHERE id=1; END;
CREATE TRIGGER search_projection_update AFTER UPDATE ON search_projection BEGIN UPDATE search_index_state SET dirty=1 WHERE id=1; END;
CREATE TRIGGER search_projection_delete AFTER DELETE ON search_projection BEGIN UPDATE search_index_state SET dirty=1 WHERE id=1; END;
UPDATE search_index_state SET dirty=1 WHERE id=1;
