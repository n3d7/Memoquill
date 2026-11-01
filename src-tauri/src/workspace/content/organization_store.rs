use super::*;
use crate::organization::*;
use serde::de::DeserializeOwned;
use std::collections::BTreeMap;

pub(super) fn load(db: &Database) -> Result<Snapshot> {
    load_connection(&db.connection, &db.id)
}
pub(super) fn load_connection(connection: &Connection, id: &WorkspaceId) -> Result<Snapshot> {
    let workspace_id = uuid(id.as_str())?;
    let revision: i64 = connection
        .query_row(
            "SELECT revision FROM library_meta WHERE workspace_id=?1",
            [id.as_str()],
            |r| r.get(0),
        )
        .optional()?
        .unwrap_or(0);
    let mut snapshot = Organization::new(workspace_id).snapshot().clone();
    snapshot.revision = integer(revision)?;
    load_subject_tree(connection, &mut snapshot)?;
    load_learning_content(connection, &mut snapshot)?;
    load_navigation(connection, &mut snapshot)?;

    Organization::from_snapshot(snapshot.clone()).map_err(|_| corrupt())?;
    Ok(snapshot)
}

fn rows<K: DeserializeOwned + Ord, T: DeserializeOwned>(
    connection: &Connection,
    query: &str,
) -> Result<BTreeMap<K, T>> {
    let values = connection
        .prepare(query)?
        .query_map([], |r| r.get::<_, String>(0))?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    let mut result = BTreeMap::new();
    for row in values {
        let value: serde_json::Value = serde_json::from_str(&row).map_err(|_| corrupt())?;
        let key = serde_json::from_value(value["id"].clone()).map_err(|_| corrupt())?;
        let record = serde_json::from_value(value).map_err(|_| corrupt())?;
        if result.insert(key, record).is_some() {
            return Err(corrupt());
        }
    }
    Ok(result)
}

pub(super) fn save(tx: &Connection, snapshot: &Snapshot) -> Result<()> {
    save_entities(tx, snapshot)?;
    save_links(tx, snapshot)?;
    tx.execute("INSERT INTO library_meta(workspace_id,revision) VALUES(?1,?2) ON CONFLICT(workspace_id) DO UPDATE SET revision=excluded.revision",params![snapshot.workspace_id.to_string(),snapshot.revision as i64])?;
    Ok(())
}
fn save_entities(tx: &Connection, s: &Snapshot) -> Result<()> {
    save_subject_tree(tx, s)?;
    save_learning_entities(tx, s)?;
    save_navigation_entities(tx, s)?;
    Ok(())
}
fn save_links(tx: &Connection, s: &Snapshot) -> Result<()> {
    tx.execute_batch("DELETE FROM placements; DELETE FROM relations; DELETE FROM unit_goals; DELETE FROM unit_resources; DELETE FROM path_steps; DELETE FROM tag_resources;")?;
    save_placements(tx, s)?;
    save_ordered_memberships(tx, s)?;
    save_path_steps(tx, s)?;
    save_relations(tx, s)?;
    Ok(())
}

fn load_subject_tree(connection: &Connection, snapshot: &mut Snapshot) -> Result<()> {
    snapshot.subjects = rows(
        connection,
        "SELECT json_object('id',id,'title',title,'priorKnowledge',prior_knowledge,'archived',json(CASE archived WHEN 1 THEN 'true' ELSE 'false' END)) FROM subjects",
    )?;
    snapshot.goals = rows(
        connection,
        "SELECT json_object('id',id,'subjectId',subject_id,'title',title,'criteria',criteria,'archived',json(CASE archived WHEN 1 THEN 'true' ELSE 'false' END)) FROM goals",
    )?;
    snapshot.collections = rows(
        connection,
        "SELECT json_object('id',id,'subjectId',subject_id,'parentId',parent_id,'title',title,'position',position,'archived',json(CASE archived WHEN 1 THEN 'true' ELSE 'false' END)) FROM collections",
    )?;
    Ok(())
}

fn load_learning_content(connection: &Connection, snapshot: &mut Snapshot) -> Result<()> {
    snapshot.resources = rows(
        connection,
        "SELECT json_object('id',id,'title',title,'archived',json(CASE archived WHEN 1 THEN 'true' ELSE 'false' END)) FROM resources",
    )?;
    snapshot.placements = rows(
        connection,
        "SELECT json_object('id',id,'collectionId',collection_id,'target',json_object('kind',CASE WHEN resource_id IS NULL THEN 'unit' ELSE 'resource' END,'id',coalesce(resource_id,unit_id)),'position',position,'archived',json(CASE archived WHEN 1 THEN 'true' ELSE 'false' END)) FROM placements",
    )?;
    snapshot.units = rows(
        connection,
        "SELECT json_object('id',id,'title',title,'objective',objective,'goalIds',json((SELECT json_group_array(goal_id) FROM (SELECT goal_id FROM unit_goals WHERE unit_id=units.id ORDER BY position))),'resourceIds',json((SELECT json_group_array(resource_id) FROM (SELECT resource_id FROM unit_resources WHERE unit_id=units.id ORDER BY position))),'archived',json(CASE archived WHEN 1 THEN 'true' ELSE 'false' END)) FROM units",
    )?;
    Ok(())
}

fn load_navigation(connection: &Connection, snapshot: &mut Snapshot) -> Result<()> {
    snapshot.paths = rows(
        connection,
        "SELECT json_object('id',id,'title',title,'steps',json((SELECT json_group_array(json_object('id',id,'unitId',unit_id)) FROM (SELECT id,unit_id FROM path_steps WHERE path_id=paths.id ORDER BY position))),'archived',json(CASE archived WHEN 1 THEN 'true' ELSE 'false' END)) FROM paths",
    )?;
    snapshot.relations = rows(
        connection,
        "SELECT json_object('id',id,'kind',kind,'from',from_unit,'to',to_unit) FROM relations",
    )?;
    snapshot.tags = rows(
        connection,
        "SELECT json_object('id',id,'title',title,'resourceIds',json((SELECT json_group_array(resource_id) FROM (SELECT resource_id FROM tag_resources WHERE tag_id=tags.id ORDER BY position)))) FROM tags",
    )?;
    Ok(())
}

fn save_subject_tree(tx: &Connection, s: &Snapshot) -> Result<()> {
    for x in s.subjects.values() {
        tx.execute("INSERT INTO subjects VALUES(?1,?2,?3,?4) ON CONFLICT(id) DO UPDATE SET title=excluded.title,prior_knowledge=excluded.prior_knowledge,archived=excluded.archived",params![x.id.to_string(),x.title,x.prior_knowledge,x.archived])?;
    }
    for x in s.goals.values() {
        tx.execute("INSERT INTO goals VALUES(?1,?2,?3,?4,?5) ON CONFLICT(id) DO UPDATE SET subject_id=excluded.subject_id,title=excluded.title,criteria=excluded.criteria,archived=excluded.archived",params![x.id.to_string(),x.subject_id.to_string(),x.title,x.criteria,x.archived])?;
    }
    for x in s.collections.values() {
        tx.execute("INSERT INTO collections VALUES(?1,?2,?3,?4,?5,?6) ON CONFLICT(id) DO UPDATE SET subject_id=excluded.subject_id,parent_id=excluded.parent_id,title=excluded.title,position=excluded.position,archived=excluded.archived",params![x.id.to_string(),x.subject_id.to_string(),x.parent_id.map(|id|id.to_string()),x.title,x.position,x.archived])?;
    }
    Ok(())
}

fn save_learning_entities(tx: &Connection, s: &Snapshot) -> Result<()> {
    for x in s.resources.values() {
        tx.execute("INSERT INTO resources VALUES(?1,?2,?3) ON CONFLICT(id) DO UPDATE SET title=excluded.title,archived=excluded.archived",params![x.id.to_string(),x.title,x.archived])?;
    }
    for x in s.units.values() {
        tx.execute("INSERT INTO units VALUES(?1,?2,?3,?4) ON CONFLICT(id) DO UPDATE SET title=excluded.title,objective=excluded.objective,archived=excluded.archived",params![x.id.to_string(),x.title,x.objective,x.archived])?;
    }
    Ok(())
}

fn save_navigation_entities(tx: &Connection, s: &Snapshot) -> Result<()> {
    for x in s.paths.values() {
        tx.execute("INSERT INTO paths VALUES(?1,?2,?3) ON CONFLICT(id) DO UPDATE SET title=excluded.title,archived=excluded.archived",params![x.id.to_string(),x.title,x.archived])?;
    }
    for x in s.tags.values() {
        tx.execute(
            "INSERT INTO tags VALUES(?1,?2) ON CONFLICT(id) DO UPDATE SET title=excluded.title",
            params![x.id.to_string(), x.title],
        )?;
    }
    Ok(())
}

fn save_placements(tx: &Connection, s: &Snapshot) -> Result<()> {
    for x in s.placements.values() {
        let (r, u) = match x.target {
            Target::Resource(id) => (Some(id.to_string()), None),
            Target::Unit(id) => (None, Some(id.to_string())),
        };
        tx.execute(
            "INSERT INTO placements VALUES(?1,?2,?3,?4,?5,?6)",
            params![
                x.id.to_string(),
                x.collection_id.to_string(),
                r,
                u,
                x.position,
                x.archived
            ],
        )?;
    }
    Ok(())
}

fn save_ordered_memberships(tx: &Connection, s: &Snapshot) -> Result<()> {
    for unit in s.units.values() {
        save_memberships(
            tx,
            "INSERT INTO unit_goals VALUES(?1,?2,?3)",
            unit.id,
            &unit.goal_ids,
        )?;
        save_memberships(
            tx,
            "INSERT INTO unit_resources VALUES(?1,?2,?3)",
            unit.id,
            &unit.resource_ids,
        )?;
    }
    for tag in s.tags.values() {
        save_memberships(
            tx,
            "INSERT INTO tag_resources VALUES(?1,?2,?3)",
            tag.id,
            &tag.resource_ids,
        )?;
    }
    Ok(())
}

fn save_path_steps(tx: &Connection, s: &Snapshot) -> Result<()> {
    for x in s.paths.values() {
        for (n, step) in x.steps.iter().enumerate() {
            tx.execute(
                "INSERT INTO path_steps VALUES(?1,?2,?3,?4)",
                params![
                    step.id.to_string(),
                    x.id.to_string(),
                    step.unit_id.to_string(),
                    n as i64
                ],
            )?;
        }
    }
    Ok(())
}

fn save_relations(tx: &Connection, s: &Snapshot) -> Result<()> {
    for x in s.relations.values() {
        let kind = match x.kind {
            RelationKind::Prerequisite => "prerequisite",
            RelationKind::Related => "related",
            RelationKind::Reference => "reference",
        };
        tx.execute(
            "INSERT INTO relations VALUES(?1,?2,?3,?4)",
            params![x.id.to_string(), kind, x.from.to_string(), x.to.to_string()],
        )?;
    }
    Ok(())
}

fn save_memberships<T: ToString>(
    tx: &Connection,
    query: &str,
    owner: impl ToString,
    members: &[T],
) -> Result<()> {
    let owner = owner.to_string();
    for (position, member) in members.iter().enumerate() {
        tx.execute(query, params![owner, member.to_string(), position as i64])?;
    }
    Ok(())
}
