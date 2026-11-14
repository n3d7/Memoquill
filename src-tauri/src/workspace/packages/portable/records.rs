//! V1 records are a fixed compile-time whitelist, never package-supplied SQL/schema.
use super::*;
use rusqlite::{
    ToSql,
    types::{FromSql, FromSqlResult, ToSqlOutput, ValueRef},
};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub(super) struct Json<T>(pub T);
impl<T: serde::de::DeserializeOwned> FromSql for Json<T> {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        serde_json::from_str(value.as_str()?)
            .map(Self)
            .map_err(|e| rusqlite::types::FromSqlError::Other(Box::new(e)))
    }
}
impl<T: Serialize> ToSql for Json<T> {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        serde_json::to_string(&self.0)
            .map(|v| ToSqlOutput::Owned(v.into()))
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))
    }
}

macro_rules! record {
    ($name:ident, $table:literal, $select:literal, $insert:literal, {$($field:ident: $ty:ty = $index:literal),+}) => {
        #[derive(Clone, Debug, Serialize, Deserialize)]
        #[serde(rename_all="camelCase", deny_unknown_fields)]
        pub(super) struct $name { $(pub $field: $ty),+ }
        impl $name {
            fn capture(c: &Connection) -> Result<Vec<Self>> {
                let mut q = c.prepare($select)?;
                let rows = q.query_map([], |r| Ok(Self { $($field: r.get($index)?),+ }))?;
                let values = rows.collect::<std::result::Result<Vec<_>,_>>()?;
                if values.len() > MAX_ROWS { return Err(budget()); }
                Ok(values)
            }
            fn insert(&self, c: &Connection) -> Result<()> {
                // IDs are canonical v4, including parent/foreign identities. Strings that
                // are semantic labels (kind/title etc.) are checked by domain validators.
                let value = serde_json::to_value(self).map_err(|_| invalid())?;
                for (key, value) in value.as_object().ok_or_else(invalid)? {
                    if key == "id" || key.ends_with("Id") || matches!(key.as_str(),"fromUnit"|"toUnit") {
                        if let Some(id) = value.as_str() {
                            if !Uuid::parse_str(id).is_ok_and(|u| u.get_version_num()==4 && u.to_string()==id) { return Err(invalid()); }
                        }
                    }
                }
                c.execute($insert, rusqlite::params![$(&self.$field),+])?;
                Ok(())
            }
        }
    }
}
record!(Subject, "subjects", "SELECT id,title,prior_knowledge,archived FROM subjects ORDER BY id,title,prior_knowledge,archived LIMIT 20001", "INSERT INTO subjects(id,title,prior_knowledge,archived) VALUES(?,?,?,?)", {id: String = 0, title: String = 1, prior_knowledge: Option<String> = 2, archived: bool = 3});
record!(Goal, "goals", "SELECT id,subject_id,title,criteria,archived FROM goals ORDER BY id,subject_id,title,criteria,archived LIMIT 20001", "INSERT INTO goals(id,subject_id,title,criteria,archived) VALUES(?,?,?,?,?)", {id: String = 0, subject_id: String = 1, title: String = 2, criteria: String = 3, archived: bool = 4});
record!(Collection, "collections", "SELECT id,subject_id,parent_id,title,position,archived FROM collections ORDER BY id,subject_id,parent_id,title,position,archived LIMIT 20001", "INSERT INTO collections(id,subject_id,parent_id,title,position,archived) VALUES(?,?,?,?,?,?)", {id: String = 0, subject_id: String = 1, parent_id: Option<String> = 2, title: String = 3, position: i64 = 4, archived: bool = 5});
record!(Resource, "resources", "SELECT id,title,archived FROM resources ORDER BY id,title,archived LIMIT 20001", "INSERT INTO resources(id,title,archived) VALUES(?,?,?)", {id: String = 0, title: String = 1, archived: bool = 2});
record!(Unit, "units", "SELECT id,title,objective,archived FROM units ORDER BY id,title,objective,archived LIMIT 20001", "INSERT INTO units(id,title,objective,archived) VALUES(?,?,?,?)", {id: String = 0, title: String = 1, objective: String = 2, archived: bool = 3});
record!(UnitGoal, "unit_goals", "SELECT unit_id,goal_id,position FROM unit_goals ORDER BY unit_id,goal_id,position LIMIT 20001", "INSERT INTO unit_goals(unit_id,goal_id,position) VALUES(?,?,?)", {unit_id: String = 0, goal_id: String = 1, position: i64 = 2});
record!(UnitResource, "unit_resources", "SELECT unit_id,resource_id,position FROM unit_resources ORDER BY unit_id,resource_id,position LIMIT 20001", "INSERT INTO unit_resources(unit_id,resource_id,position) VALUES(?,?,?)", {unit_id: String = 0, resource_id: String = 1, position: i64 = 2});
record!(Placement, "placements", "SELECT id,collection_id,resource_id,unit_id,position,archived FROM placements ORDER BY id,collection_id,resource_id,unit_id,position,archived LIMIT 20001", "INSERT INTO placements(id,collection_id,resource_id,unit_id,position,archived) VALUES(?,?,?,?,?,?)", {id: String = 0, collection_id: String = 1, resource_id: Option<String> = 2, unit_id: Option<String> = 3, position: i64 = 4, archived: bool = 5});
record!(LearningPath, "paths", "SELECT id,title,archived FROM paths ORDER BY id,title,archived LIMIT 20001", "INSERT INTO paths(id,title,archived) VALUES(?,?,?)", {id: String = 0, title: String = 1, archived: bool = 2});
record!(PathStep, "path_steps", "SELECT id,path_id,unit_id,position FROM path_steps ORDER BY id,path_id,unit_id,position LIMIT 20001", "INSERT INTO path_steps(id,path_id,unit_id,position) VALUES(?,?,?,?)", {id: String = 0, path_id: String = 1, unit_id: String = 2, position: i64 = 3});
record!(Relation, "relations", "SELECT id,kind,from_unit,to_unit FROM relations ORDER BY id,kind,from_unit,to_unit LIMIT 20001", "INSERT INTO relations(id,kind,from_unit,to_unit) VALUES(?,?,?,?)", {id: String = 0, kind: String = 1, from_unit: String = 2, to_unit: String = 3});
record!(Tag, "tags", "SELECT id,title FROM tags ORDER BY id,title LIMIT 20001", "INSERT INTO tags(id,title) VALUES(?,?)", {id: String = 0, title: String = 1});
record!(TagResource, "tag_resources", "SELECT tag_id,resource_id,position FROM tag_resources ORDER BY tag_id,resource_id,position LIMIT 20001", "INSERT INTO tag_resources(tag_id,resource_id,position) VALUES(?,?,?)", {tag_id: String = 0, resource_id: String = 1, position: i64 = 2});
record!(ResourceRevisionRow, "resource_revisions", "SELECT id,resource_id,revision,document,digest FROM resource_revisions ORDER BY id,resource_id,revision,document,digest LIMIT 20001", "INSERT INTO resource_revisions(id,resource_id,revision,document,digest) VALUES(?,?,?,?,?)", {id: String = 0, resource_id: String = 1, revision: i64 = 2, document: Json<crate::document::Document> = 3, digest: String = 4});
record!(ResourceHead, "resource_heads", "SELECT resource_id,revision_id FROM resource_heads ORDER BY resource_id,revision_id LIMIT 20001", "INSERT INTO resource_heads(resource_id,revision_id) VALUES(?,?)", {resource_id: String = 0, revision_id: String = 1});
record!(SourceCounter, "source_revisions", "SELECT resource_id,revision FROM source_revisions ORDER BY resource_id,revision LIMIT 20001", "INSERT INTO source_revisions(resource_id,revision) VALUES(?,?)", {resource_id: String = 0, revision: i64 = 1});
record!(Source, "original_sources", "SELECT id,resource_id,digest,filename,media_type,attribution,license,origin_url,revision,byte_length FROM original_sources ORDER BY id,resource_id,digest,filename,media_type,attribution,license,origin_url,revision,byte_length LIMIT 20001", "INSERT INTO original_sources(id,resource_id,digest,filename,media_type,attribution,license,origin_url,revision,byte_length) VALUES(?,?,?,?,?,?,?,?,?,?)", {id: String = 0, resource_id: String = 1, digest: String = 2, filename: String = 3, media_type: String = 4, attribution: Option<String> = 5, license: Option<String> = 6, origin_url: Option<String> = 7, revision: i64 = 8, byte_length: i64 = 9});
record!(LessonRevision, "lesson_revisions", "SELECT id,unit_id,parent_id,data,digest FROM lesson_revisions ORDER BY id,unit_id,parent_id,data,digest LIMIT 20001", "INSERT INTO lesson_revisions(id,unit_id,parent_id,data,digest) VALUES(?,?,?,?,?)", {id: String = 0, unit_id: String = 1, parent_id: Option<String> = 2, data: Json<Composition> = 3, digest: String = 4});
record!(LessonHead, "lesson_heads", "SELECT unit_id,revision_id FROM lesson_heads ORDER BY unit_id,revision_id LIMIT 20001", "INSERT INTO lesson_heads(unit_id,revision_id) VALUES(?,?)", {unit_id: String = 0, revision_id: String = 1});
record!(ActivityRevisionRow, "lesson_activity_revisions", "SELECT id,activity_id,unit_id,definition,digest FROM lesson_activity_revisions ORDER BY id,activity_id,unit_id,definition,digest LIMIT 20001", "INSERT INTO lesson_activity_revisions(id,activity_id,unit_id,definition,digest) VALUES(?,?,?,?,?)", {id: String = 0, activity_id: String = 1, unit_id: String = 2, definition: Json<serde_json::Value> = 3, digest: String = 4});
record!(ActivityHead, "lesson_activity_heads", "SELECT activity_id,unit_id,revision_id FROM lesson_activity_heads ORDER BY activity_id,unit_id,revision_id LIMIT 20001", "INSERT INTO lesson_activity_heads(activity_id,unit_id,revision_id) VALUES(?,?,?)", {activity_id: String = 0, unit_id: String = 1, revision_id: String = 2});
record!(ResourceOwner, "lesson_resource_owners", "SELECT resource_id,unit_id FROM lesson_resource_owners ORDER BY resource_id,unit_id LIMIT 20001", "INSERT INTO lesson_resource_owners(resource_id,unit_id) VALUES(?,?)", {resource_id: String = 0, unit_id: String = 1});
record!(Dependency, "lesson_dependencies", "SELECT composition_id,kind,target_id,revision_id FROM lesson_dependencies ORDER BY composition_id,kind,target_id,revision_id LIMIT 20001", "INSERT INTO lesson_dependencies(composition_id,kind,target_id,revision_id) VALUES(?,?,?,?)", {composition_id: String = 0, kind: String = 1, target_id: String = 2, revision_id: String = 3});
record!(SourceBinding, "lesson_source_bindings", "SELECT id,unit_id,source_id,data,digest FROM lesson_source_bindings ORDER BY id,unit_id,source_id,data,digest LIMIT 20001", "INSERT INTO lesson_source_bindings(id,unit_id,source_id,data,digest) VALUES(?,?,?,?,?)", {id: String = 0, unit_id: String = 1, source_id: String = 2, data: Json<SourceBindingRevision> = 3, digest: String = 4});

// Preserve JSON bytes and digests exactly, including legacy unknown metadata.
record!(AssessmentRunRow,"assessment_runs","SELECT id,operation_id,request,request_hash,data,digest FROM assessment_runs ORDER BY rowid LIMIT 20001","INSERT INTO assessment_runs(id,operation_id,request,request_hash,data,digest) VALUES(?,?,?,?,?,?)",{id:String=0,operation_id:String=1,request:String=2,request_hash:String=3,data:String=4,digest:String=5});
record!(AssessmentPartRow,"assessment_parts","SELECT session_id,assessment_id,part_index,unit_id,composition_id FROM assessment_parts ORDER BY rowid LIMIT 20001","INSERT INTO assessment_parts(session_id,assessment_id,part_index,unit_id,composition_id) VALUES(?,?,?,?,?)",{session_id:String=0,assessment_id:String=1,part_index:i64=2,unit_id:String=3,composition_id:String=4});
record!(ReviewJournalRow,"review_journal","SELECT position,kind,record_id FROM review_journal ORDER BY position LIMIT 20001","INSERT INTO review_journal(position,kind,record_id) VALUES(?,?,?)",{position:i64=0,kind:String=1,record_id:String=2});
record!(ReviewPlanRow,"review_plan","SELECT id,data,digest FROM review_plan ORDER BY rowid LIMIT 20001","INSERT INTO review_plan(id,data,digest) VALUES(?,?,?)",{id:i64=0,data:String=1,digest:String=2});
record!(ReviewOperationRow,"review_operations","SELECT id,request,request_hash,recorded_at_ms,result,digest FROM review_operations ORDER BY rowid LIMIT 20001","INSERT INTO review_operations(id,request,request_hash,recorded_at_ms,result,digest) VALUES(?,?,?,?,?,?)",{id:String=0,request:String=1,request_hash:String=2,recorded_at_ms:i64=3,result:String=4,digest:String=5});
record!(RecallEventRow,"recall_review_events","SELECT id,attempt_id,activity_revision_id,prior_event_id,reviewed_at_ms,study_day,data,digest FROM recall_review_events ORDER BY rowid LIMIT 20001","INSERT INTO recall_review_events(id,attempt_id,activity_revision_id,prior_event_id,reviewed_at_ms,study_day,data,digest) VALUES(?,?,?,?,?,?,?,?)",{id:String=0,attempt_id:String=1,activity_revision_id:String=2,prior_event_id:Option<String>=3,reviewed_at_ms:i64=4,study_day:i32=5,data:String=6,digest:String=7});
record!(RecallStateRow,"recall_states","SELECT activity_revision_id,event_id FROM recall_states ORDER BY rowid LIMIT 20001","INSERT INTO recall_states(activity_revision_id,event_id) VALUES(?,?)",{activity_revision_id:String=0,event_id:String=1});
record!(StudySessionRow, "study_sessions", "SELECT id,unit_id,composition_id,revision,data,digest FROM study_sessions ORDER BY rowid LIMIT 20001", "INSERT INTO study_sessions(id,unit_id,composition_id,revision,data,digest) VALUES(?,?,?,?,?,?)", {id: String = 0, unit_id: String = 1, composition_id: String = 2, revision: i64 = 3, data: String = 4, digest: String = 5});
record!(StudyAttemptRow, "study_attempts", "SELECT id,session_id,item_index,data,digest FROM study_attempts ORDER BY rowid LIMIT 20001", "INSERT INTO study_attempts(id,session_id,item_index,data,digest) VALUES(?,?,?,?,?)", {id: String = 0, session_id: String = 1, item_index: i64 = 2, data: String = 3, digest: String = 4});
record!(StudyJudgmentRow, "study_judgments", "SELECT id,attempt_id,data,digest FROM study_judgments ORDER BY rowid LIMIT 20001", "INSERT INTO study_judgments(id,attempt_id,data,digest) VALUES(?,?,?,?)", {id: String = 0, attempt_id: String = 1, data: String = 2, digest: String = 3});
record!(StudyOperationRow, "study_operations", "SELECT id,session_id,request_hash,result,digest FROM study_operations ORDER BY rowid LIMIT 20001", "INSERT INTO study_operations(id,session_id,request_hash,result,digest) VALUES(?,?,?,?,?)", {id: String = 0, session_id: String = 1, request_hash: String = 2, result: String = 3, digest: String = 4});

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct Content {
    pub subjects: Vec<Subject>,
    pub goals: Vec<Goal>,
    pub collections: Vec<Collection>,
    pub resources: Vec<Resource>,
    pub units: Vec<Unit>,
    pub unit_goals: Vec<UnitGoal>,
    pub unit_resources: Vec<UnitResource>,
    pub placements: Vec<Placement>,
    pub paths: Vec<LearningPath>,
    pub path_steps: Vec<PathStep>,
    pub relations: Vec<Relation>,
    pub tags: Vec<Tag>,
    pub tag_resources: Vec<TagResource>,
    pub resource_revisions: Vec<ResourceRevisionRow>,
    pub resource_heads: Vec<ResourceHead>,
    pub source_revisions: Vec<SourceCounter>,
    pub original_sources: Vec<Source>,
    pub lesson_revisions: Vec<LessonRevision>,
    pub lesson_heads: Vec<LessonHead>,
    pub lesson_activity_revisions: Vec<ActivityRevisionRow>,
    pub lesson_activity_heads: Vec<ActivityHead>,
    pub lesson_resource_owners: Vec<ResourceOwner>,
    pub lesson_dependencies: Vec<Dependency>,
    pub lesson_source_bindings: Vec<SourceBinding>,
    pub assets: Vec<Asset>,
    pub review_journal: Vec<ReviewJournalRow>,
    pub assessment_runs: Vec<AssessmentRunRow>,
    pub assessment_parts: Vec<AssessmentPartRow>,
    pub review_plan: Vec<ReviewPlanRow>,
    pub review_operations: Vec<ReviewOperationRow>,
    pub recall_review_events: Vec<RecallEventRow>,
    pub recall_states: Vec<RecallStateRow>,
    pub study_sessions: Vec<StudySessionRow>,
    pub study_attempts: Vec<StudyAttemptRow>,
    pub study_judgments: Vec<StudyJudgmentRow>,
    pub study_operations: Vec<StudyOperationRow>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct Asset {
    pub digest: AssetDigest,
    pub byte_length: u64,
}
impl Content {
    pub fn capture(c: &Connection) -> Result<Self> {
        Ok(Self {
            subjects: Subject::capture(c)?,
            goals: Goal::capture(c)?,
            collections: Collection::capture(c)?,
            resources: Resource::capture(c)?,
            units: Unit::capture(c)?,
            unit_goals: UnitGoal::capture(c)?,
            unit_resources: UnitResource::capture(c)?,
            placements: Placement::capture(c)?,
            paths: LearningPath::capture(c)?,
            path_steps: PathStep::capture(c)?,
            relations: Relation::capture(c)?,
            tags: Tag::capture(c)?,
            tag_resources: TagResource::capture(c)?,
            resource_revisions: ResourceRevisionRow::capture(c)?,
            resource_heads: ResourceHead::capture(c)?,
            source_revisions: SourceCounter::capture(c)?,
            original_sources: Source::capture(c)?,
            lesson_revisions: LessonRevision::capture(c)?,
            lesson_heads: LessonHead::capture(c)?,
            lesson_activity_revisions: ActivityRevisionRow::capture(c)?,
            lesson_activity_heads: ActivityHead::capture(c)?,
            lesson_resource_owners: ResourceOwner::capture(c)?,
            lesson_dependencies: Dependency::capture(c)?,
            lesson_source_bindings: SourceBinding::capture(c)?,
            assets: Vec::new(),
            review_journal: Vec::new(),
            assessment_runs: Vec::new(),
            assessment_parts: Vec::new(),
            review_plan: Vec::new(),
            review_operations: Vec::new(),
            recall_review_events: Vec::new(),
            recall_states: Vec::new(),
            study_sessions: Vec::new(),
            study_attempts: Vec::new(),
            study_judgments: Vec::new(),
            study_operations: Vec::new(),
        })
    }
    pub fn capture_history(&mut self, c: &Connection) -> Result<()> {
        self.review_journal = ReviewJournalRow::capture(c)?;
        self.assessment_runs = AssessmentRunRow::capture(c)?;
        self.assessment_parts = AssessmentPartRow::capture(c)?;
        self.review_plan = ReviewPlanRow::capture(c)?;
        self.review_operations = ReviewOperationRow::capture(c)?;
        self.recall_review_events = RecallEventRow::capture(c)?;
        self.recall_states = RecallStateRow::capture(c)?;
        self.study_sessions = StudySessionRow::capture(c)?;
        self.study_attempts = StudyAttemptRow::capture(c)?;
        self.study_judgments = StudyJudgmentRow::capture(c)?;
        self.study_operations = StudyOperationRow::capture(c)?;
        Ok(())
    }
    pub fn has_review(&self) -> bool {
        !self.assessment_runs.is_empty()
            || !self.assessment_parts.is_empty()
            || !self.review_journal.is_empty()
            || !self.review_plan.is_empty()
            || !self.review_operations.is_empty()
            || !self.recall_review_events.is_empty()
            || !self.recall_states.is_empty()
    }
    pub fn has_history(&self) -> bool {
        self.has_review()
            || !self.study_sessions.is_empty()
            || !self.study_attempts.is_empty()
            || !self.study_judgments.is_empty()
            || !self.study_operations.is_empty()
    }
    pub fn insert(&self, c: &Connection) -> Result<()> {
        for row in &self.subjects {
            row.insert(c)?;
        }
        for row in &self.goals {
            row.insert(c)?;
        }
        for row in &self.collections {
            row.insert(c)?;
        }
        for row in &self.resources {
            row.insert(c)?;
        }
        for row in &self.units {
            row.insert(c)?;
        }
        for row in &self.unit_goals {
            row.insert(c)?;
        }
        for row in &self.unit_resources {
            row.insert(c)?;
        }
        for row in &self.placements {
            row.insert(c)?;
        }
        for row in &self.paths {
            row.insert(c)?;
        }
        for row in &self.path_steps {
            row.insert(c)?;
        }
        for row in &self.relations {
            row.insert(c)?;
        }
        for row in &self.tags {
            row.insert(c)?;
        }
        for row in &self.tag_resources {
            row.insert(c)?;
        }
        for row in &self.resource_revisions {
            row.insert(c)?;
        }
        for row in &self.resource_heads {
            row.insert(c)?;
        }
        for row in &self.source_revisions {
            row.insert(c)?;
        }
        for row in &self.original_sources {
            row.insert(c)?;
        }
        for row in &self.lesson_revisions {
            row.insert(c)?;
        }
        for row in &self.lesson_heads {
            row.insert(c)?;
        }
        for row in &self.lesson_activity_revisions {
            row.insert(c)?;
        }
        for row in &self.lesson_activity_heads {
            row.insert(c)?;
        }
        for row in &self.lesson_resource_owners {
            row.insert(c)?;
        }
        for row in &self.lesson_dependencies {
            row.insert(c)?;
        }
        for row in &self.lesson_source_bindings {
            row.insert(c)?;
        }
        for row in &self.study_sessions {
            row.insert(c)?;
        }
        for row in &self.study_attempts {
            row.insert(c)?;
        }
        for row in &self.study_judgments {
            row.insert(c)?;
        }
        for row in &self.study_operations {
            row.insert(c)?;
        }
        for row in &self.review_plan {
            row.insert(c)?;
        }
        for row in &self.review_operations {
            row.insert(c)?;
        }
        for row in &self.recall_review_events {
            row.insert(c)?;
        }
        for row in &self.recall_states {
            row.insert(c)?;
        }
        for row in &self.review_journal {
            row.insert(c)?;
        }
        for row in &self.assessment_runs {
            row.insert(c)?;
        }
        for row in &self.assessment_parts {
            row.insert(c)?;
        }
        Ok(())
    }
    pub fn counts(&self) -> Result<BTreeMap<String, u64>> {
        let mut counts = BTreeMap::from([
            ("subjects".into(), self.subjects.len() as u64),
            ("goals".into(), self.goals.len() as u64),
            ("collections".into(), self.collections.len() as u64),
            ("resources".into(), self.resources.len() as u64),
            ("units".into(), self.units.len() as u64),
            ("unit_goals".into(), self.unit_goals.len() as u64),
            ("unit_resources".into(), self.unit_resources.len() as u64),
            ("placements".into(), self.placements.len() as u64),
            ("paths".into(), self.paths.len() as u64),
            ("path_steps".into(), self.path_steps.len() as u64),
            ("relations".into(), self.relations.len() as u64),
            ("tags".into(), self.tags.len() as u64),
            ("tag_resources".into(), self.tag_resources.len() as u64),
            (
                "resource_revisions".into(),
                self.resource_revisions.len() as u64,
            ),
            ("resource_heads".into(), self.resource_heads.len() as u64),
            (
                "source_revisions".into(),
                self.source_revisions.len() as u64,
            ),
            (
                "original_sources".into(),
                self.original_sources.len() as u64,
            ),
            (
                "lesson_revisions".into(),
                self.lesson_revisions.len() as u64,
            ),
            ("lesson_heads".into(), self.lesson_heads.len() as u64),
            (
                "lesson_activity_revisions".into(),
                self.lesson_activity_revisions.len() as u64,
            ),
            (
                "lesson_activity_heads".into(),
                self.lesson_activity_heads.len() as u64,
            ),
            (
                "lesson_resource_owners".into(),
                self.lesson_resource_owners.len() as u64,
            ),
            (
                "lesson_dependencies".into(),
                self.lesson_dependencies.len() as u64,
            ),
            (
                "lesson_source_bindings".into(),
                self.lesson_source_bindings.len() as u64,
            ),
            ("assets".into(), self.assets.len() as u64),
        ]);
        if self.has_history() {
            counts.insert("study_sessions".into(), self.study_sessions.len() as u64);
            counts.insert("study_attempts".into(), self.study_attempts.len() as u64);
            counts.insert("study_judgments".into(), self.study_judgments.len() as u64);
            counts.insert(
                "study_operations".into(),
                self.study_operations.len() as u64,
            );
        }
        if self.has_review() {
            counts.insert("assessment_runs".into(), self.assessment_runs.len() as u64);
            counts.insert(
                "assessment_parts".into(),
                self.assessment_parts.len() as u64,
            );
            counts.insert("review_journal".into(), self.review_journal.len() as u64);
            counts.insert("review_plan".into(), self.review_plan.len() as u64);
            counts.insert(
                "review_operations".into(),
                self.review_operations.len() as u64,
            );
            counts.insert(
                "recall_review_events".into(),
                self.recall_review_events.len() as u64,
            );
            counts.insert("recall_states".into(), self.recall_states.len() as u64);
        }
        if counts.values().any(|&n| n > MAX_ROWS as u64) || counts.values().sum::<u64>() > 100_000 {
            return Err(budget());
        }
        Ok(counts)
    }
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "family", content = "record", deny_unknown_fields)]
enum Record {
    #[serde(rename = "assessment_runs")]
    AssessmentRunRow(AssessmentRunRow),
    #[serde(rename = "assessment_parts")]
    AssessmentPartRow(AssessmentPartRow),
    #[serde(rename = "review_journal")]
    ReviewJournalRow(ReviewJournalRow),
    #[serde(rename = "review_plan")]
    ReviewPlanRow(ReviewPlanRow),
    #[serde(rename = "review_operations")]
    ReviewOperationRow(ReviewOperationRow),
    #[serde(rename = "recall_review_events")]
    RecallEventRow(RecallEventRow),
    #[serde(rename = "recall_states")]
    RecallStateRow(RecallStateRow),
    #[serde(rename = "study_sessions")]
    StudySessionRow(StudySessionRow),
    #[serde(rename = "study_attempts")]
    StudyAttemptRow(StudyAttemptRow),
    #[serde(rename = "study_judgments")]
    StudyJudgmentRow(StudyJudgmentRow),
    #[serde(rename = "study_operations")]
    StudyOperationRow(StudyOperationRow),

    #[serde(rename = "subjects")]
    Subject(Subject),
    #[serde(rename = "goals")]
    Goal(Goal),
    #[serde(rename = "collections")]
    Collection(Collection),
    #[serde(rename = "resources")]
    Resource(Resource),
    #[serde(rename = "units")]
    Unit(Unit),
    #[serde(rename = "unit_goals")]
    UnitGoal(UnitGoal),
    #[serde(rename = "unit_resources")]
    UnitResource(UnitResource),
    #[serde(rename = "placements")]
    Placement(Placement),
    #[serde(rename = "paths")]
    LearningPath(LearningPath),
    #[serde(rename = "path_steps")]
    PathStep(PathStep),
    #[serde(rename = "relations")]
    Relation(Relation),
    #[serde(rename = "tags")]
    Tag(Tag),
    #[serde(rename = "tag_resources")]
    TagResource(TagResource),
    #[serde(rename = "resource_revisions")]
    ResourceRevisionRow(ResourceRevisionRow),
    #[serde(rename = "resource_heads")]
    ResourceHead(ResourceHead),
    #[serde(rename = "source_revisions")]
    SourceCounter(SourceCounter),
    #[serde(rename = "original_sources")]
    Source(Source),
    #[serde(rename = "lesson_revisions")]
    LessonRevision(LessonRevision),
    #[serde(rename = "lesson_heads")]
    LessonHead(LessonHead),
    #[serde(rename = "lesson_activity_revisions")]
    ActivityRevisionRow(ActivityRevisionRow),
    #[serde(rename = "lesson_activity_heads")]
    ActivityHead(ActivityHead),
    #[serde(rename = "lesson_resource_owners")]
    ResourceOwner(ResourceOwner),
    #[serde(rename = "lesson_dependencies")]
    Dependency(Dependency),
    #[serde(rename = "lesson_source_bindings")]
    SourceBinding(SourceBinding),
    #[serde(rename = "assets")]
    Asset(Asset),
}
impl Content {
    pub fn encode(&self) -> Result<Vec<u8>> {
        fn push(out: &mut Vec<u8>, row: Record) -> Result<()> {
            let data = json(&row)?;
            if data.len() > 2 * 1024 * 1024 {
                return Err(budget());
            }
            out.extend(data);
            out.push(b'\n');
            if out.len() as u64 > MAX_CONTENT {
                return Err(budget());
            }
            Ok(())
        }
        let mut out = Vec::new();
        for row in &self.subjects {
            push(&mut out, Record::Subject(row.clone()))?;
        }
        for row in &self.goals {
            push(&mut out, Record::Goal(row.clone()))?;
        }
        for row in &self.collections {
            push(&mut out, Record::Collection(row.clone()))?;
        }
        for row in &self.resources {
            push(&mut out, Record::Resource(row.clone()))?;
        }
        for row in &self.units {
            push(&mut out, Record::Unit(row.clone()))?;
        }
        for row in &self.unit_goals {
            push(&mut out, Record::UnitGoal(row.clone()))?;
        }
        for row in &self.unit_resources {
            push(&mut out, Record::UnitResource(row.clone()))?;
        }
        for row in &self.placements {
            push(&mut out, Record::Placement(row.clone()))?;
        }
        for row in &self.paths {
            push(&mut out, Record::LearningPath(row.clone()))?;
        }
        for row in &self.path_steps {
            push(&mut out, Record::PathStep(row.clone()))?;
        }
        for row in &self.relations {
            push(&mut out, Record::Relation(row.clone()))?;
        }
        for row in &self.tags {
            push(&mut out, Record::Tag(row.clone()))?;
        }
        for row in &self.tag_resources {
            push(&mut out, Record::TagResource(row.clone()))?;
        }
        for row in &self.resource_revisions {
            push(&mut out, Record::ResourceRevisionRow(row.clone()))?;
        }
        for row in &self.resource_heads {
            push(&mut out, Record::ResourceHead(row.clone()))?;
        }
        for row in &self.source_revisions {
            push(&mut out, Record::SourceCounter(row.clone()))?;
        }
        for row in &self.original_sources {
            push(&mut out, Record::Source(row.clone()))?;
        }
        for row in &self.lesson_revisions {
            push(&mut out, Record::LessonRevision(row.clone()))?;
        }
        for row in &self.lesson_heads {
            push(&mut out, Record::LessonHead(row.clone()))?;
        }
        for row in &self.lesson_activity_revisions {
            push(&mut out, Record::ActivityRevisionRow(row.clone()))?;
        }
        for row in &self.lesson_activity_heads {
            push(&mut out, Record::ActivityHead(row.clone()))?;
        }
        for row in &self.lesson_resource_owners {
            push(&mut out, Record::ResourceOwner(row.clone()))?;
        }
        for row in &self.lesson_dependencies {
            push(&mut out, Record::Dependency(row.clone()))?;
        }
        for row in &self.lesson_source_bindings {
            push(&mut out, Record::SourceBinding(row.clone()))?;
        }
        for row in &self.assets {
            push(&mut out, Record::Asset(row.clone()))?;
        }
        for row in &self.study_sessions {
            push(&mut out, Record::StudySessionRow(row.clone()))?;
        }
        for row in &self.study_attempts {
            push(&mut out, Record::StudyAttemptRow(row.clone()))?;
        }
        for row in &self.study_judgments {
            push(&mut out, Record::StudyJudgmentRow(row.clone()))?;
        }
        for row in &self.study_operations {
            push(&mut out, Record::StudyOperationRow(row.clone()))?;
        }
        for row in &self.review_plan {
            push(&mut out, Record::ReviewPlanRow(row.clone()))?;
        }
        for row in &self.review_operations {
            push(&mut out, Record::ReviewOperationRow(row.clone()))?;
        }
        for row in &self.recall_review_events {
            push(&mut out, Record::RecallEventRow(row.clone()))?;
        }
        for row in &self.recall_states {
            push(&mut out, Record::RecallStateRow(row.clone()))?;
        }
        for row in &self.review_journal {
            push(&mut out, Record::ReviewJournalRow(row.clone()))?;
        }
        for row in &self.assessment_runs {
            push(&mut out, Record::AssessmentRunRow(row.clone()))?;
        }
        for row in &self.assessment_parts {
            push(&mut out, Record::AssessmentPartRow(row.clone()))?;
        }
        Ok(out)
    }
    pub fn decode(bytes: &[u8]) -> Result<Self> {
        if bytes.len() as u64 > MAX_CONTENT {
            return Err(budget());
        }
        let mut out = Self::default();
        let mut count = 0;
        if !bytes.is_empty() && !bytes.ends_with(b"\n") {
            return Err(invalid());
        }
        for line in bytes
            .strip_suffix(b"\n")
            .unwrap_or(bytes)
            .split(|b| *b == b'\n')
            .filter(|line| !bytes.is_empty() || !line.is_empty())
        {
            count += 1;
            if count > 100_000 || line.len() > 2 * 1024 * 1024 {
                return Err(budget());
            }
            let record: Record = serde_json::from_slice(line).map_err(|_| invalid())?;
            match record {
                Record::AssessmentRunRow(row) => out.assessment_runs.push(row),
                Record::AssessmentPartRow(row) => out.assessment_parts.push(row),
                Record::ReviewJournalRow(row) => out.review_journal.push(row),
                Record::ReviewPlanRow(row) => out.review_plan.push(row),
                Record::ReviewOperationRow(row) => out.review_operations.push(row),
                Record::RecallEventRow(row) => out.recall_review_events.push(row),
                Record::RecallStateRow(row) => out.recall_states.push(row),
                Record::StudySessionRow(row) => out.study_sessions.push(row),
                Record::StudyAttemptRow(row) => out.study_attempts.push(row),
                Record::StudyJudgmentRow(row) => out.study_judgments.push(row),
                Record::StudyOperationRow(row) => out.study_operations.push(row),

                Record::Subject(row) => out.subjects.push(row),
                Record::Goal(row) => out.goals.push(row),
                Record::Collection(row) => out.collections.push(row),
                Record::Resource(row) => out.resources.push(row),
                Record::Unit(row) => out.units.push(row),
                Record::UnitGoal(row) => out.unit_goals.push(row),
                Record::UnitResource(row) => out.unit_resources.push(row),
                Record::Placement(row) => out.placements.push(row),
                Record::LearningPath(row) => out.paths.push(row),
                Record::PathStep(row) => out.path_steps.push(row),
                Record::Relation(row) => out.relations.push(row),
                Record::Tag(row) => out.tags.push(row),
                Record::TagResource(row) => out.tag_resources.push(row),
                Record::ResourceRevisionRow(row) => out.resource_revisions.push(row),
                Record::ResourceHead(row) => out.resource_heads.push(row),
                Record::SourceCounter(row) => out.source_revisions.push(row),
                Record::Source(row) => out.original_sources.push(row),
                Record::LessonRevision(row) => out.lesson_revisions.push(row),
                Record::LessonHead(row) => out.lesson_heads.push(row),
                Record::ActivityRevisionRow(row) => out.lesson_activity_revisions.push(row),
                Record::ActivityHead(row) => out.lesson_activity_heads.push(row),
                Record::ResourceOwner(row) => out.lesson_resource_owners.push(row),
                Record::Dependency(row) => out.lesson_dependencies.push(row),
                Record::SourceBinding(row) => out.lesson_source_bindings.push(row),
                Record::Asset(row) => out.assets.push(row),
            }
        }
        out.counts()?;
        Ok(out)
    }
}
