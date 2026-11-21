CREATE TABLE agent_course_drafts (
    id TEXT PRIMARY KEY,
    actor TEXT NOT NULL,
    data TEXT NOT NULL CHECK(json_valid(data) AND length(CAST(data AS BLOB)) <= 2097152)
) STRICT;
CREATE INDEX agent_course_drafts_owner ON agent_course_drafts(actor, id);
CREATE TABLE agent_authored_documents (
    revision_id TEXT PRIMARY KEY REFERENCES resource_revisions(id),
    proposal_id TEXT NOT NULL REFERENCES agent_course_drafts(id)
) STRICT;
