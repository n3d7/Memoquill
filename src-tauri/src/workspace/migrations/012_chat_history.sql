CREATE TABLE chat_threads (
    sequence INTEGER PRIMARY KEY CHECK(sequence > 0),
    id TEXT NOT NULL UNIQUE CHECK(length(id) = 36),
    title TEXT NOT NULL CHECK(length(CAST(title AS BLOB)) BETWEEN 1 AND 256)
) STRICT;

CREATE TABLE chat_runs (
    sequence INTEGER PRIMARY KEY CHECK(sequence > 0),
    id TEXT NOT NULL UNIQUE CHECK(length(id) = 36),
    thread_id TEXT NOT NULL REFERENCES chat_threads(id),
    prompt TEXT NOT NULL CHECK(length(CAST(prompt AS BLOB)) BETWEEN 1 AND 32768),
    status TEXT NOT NULL CHECK(status IN ('streaming', 'completed', 'cancelled', 'failed', 'interrupted')),
    response TEXT CHECK(length(CAST(response AS BLOB)) <= 262144),
    CHECK((status = 'completed') = (response IS NOT NULL))
) STRICT;
CREATE INDEX chat_runs_thread_sequence ON chat_runs(thread_id, sequence);
CREATE UNIQUE INDEX chat_runs_one_stream ON chat_runs(thread_id) WHERE status = 'streaming';
