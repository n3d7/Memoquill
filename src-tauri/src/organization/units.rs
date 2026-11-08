use super::collections::active;
use super::validation::exists;
use super::*;
use std::collections::BTreeSet;

impl Organization {
    pub(super) fn apply_units(&mut self, command: Command) -> Result<(), DomainError> {
        match command {
            Command::CreateUnit {
                id,
                title,
                objective,
                goal_ids,
                resource_ids,
            } => self.create_unit(Unit {
                id,
                title,
                objective,
                goal_ids,
                resource_ids,
                archived: false,
            }),
            Command::UpdateUnit {
                id,
                title,
                objective,
                goal_ids,
                resource_ids,
            } => self.update_unit(Unit {
                id,
                title,
                objective,
                goal_ids,
                resource_ids,
                archived: false,
            }),
            Command::CreatePath { id, title, steps } => self.create_path(id, title, steps),
            Command::SetPathSteps { id, steps } => self.set_path_steps(id, steps),
            other => self.apply_relations(other),
        }
    }
    fn create_unit(&mut self, unit: Unit) -> Result<(), DomainError> {
        self.active_unit_refs(&unit.goal_ids, &unit.resource_ids)?;
        insert(&mut self.0.units, unit.id, unit)
    }
    fn update_unit(&mut self, replacement: Unit) -> Result<(), DomainError> {
        self.active_unit_refs(&replacement.goal_ids, &replacement.resource_ids)?;
        let unit = self
            .0
            .units
            .get_mut(&replacement.id)
            .ok_or(DomainError::NotFound)?;
        active(unit.archived)?;
        *unit = replacement;
        Ok(())
    }
    fn create_path(
        &mut self,
        id: PathId,
        title: String,
        steps: Vec<PathStep>,
    ) -> Result<(), DomainError> {
        self.active_steps(&steps)?;
        insert(
            &mut self.0.paths,
            id,
            Path {
                id,
                title,
                steps,
                archived: false,
            },
        )
    }
    fn set_path_steps(&mut self, id: PathId, steps: Vec<PathStep>) -> Result<(), DomainError> {
        self.active_steps(&steps)?;
        let path = self.0.paths.get_mut(&id).ok_or(DomainError::NotFound)?;
        active(path.archived)?;
        path.steps = steps;
        Ok(())
    }
    fn active_unit_refs(
        &self,
        goals: &[GoalId],
        resources: &[ResourceId],
    ) -> Result<(), DomainError> {
        for id in goals {
            let goal = self.0.goals.get(id).ok_or(DomainError::NotFound)?;
            active(goal.archived)?;
            self.active_subject(goal.subject_id)?;
        }
        for id in resources {
            active(
                self.0
                    .resources
                    .get(id)
                    .ok_or(DomainError::NotFound)?
                    .archived,
            )?;
        }
        Ok(())
    }
    fn active_steps(&self, steps: &[PathStep]) -> Result<(), DomainError> {
        for step in steps {
            active(
                self.0
                    .units
                    .get(&step.unit_id)
                    .ok_or(DomainError::NotFound)?
                    .archived,
            )?;
        }
        Ok(())
    }
    fn apply_relations(&mut self, command: Command) -> Result<(), DomainError> {
        match command {
            Command::Relate { id, kind, from, to } => self.relate(Relation { id, kind, from, to }),
            Command::RemoveRelation { id } => self
                .0
                .relations
                .remove(&id)
                .map(|_| ())
                .ok_or(DomainError::NotFound),
            Command::SetTag {
                id,
                title,
                resource_ids,
            } => self.set_tag(Tag {
                id,
                title,
                resource_ids,
            }),
            Command::Archive { target, archived } => self.archive(target, archived),
            _ => Err(DomainError::InvalidRequest),
        }
    }
    fn relate(&mut self, relation: Relation) -> Result<(), DomainError> {
        active(
            self.0
                .units
                .get(&relation.from)
                .ok_or(DomainError::NotFound)?
                .archived,
        )?;
        active(
            self.0
                .units
                .get(&relation.to)
                .ok_or(DomainError::NotFound)?
                .archived,
        )?;
        insert(&mut self.0.relations, relation.id, relation)
    }
    fn set_tag(&mut self, tag: Tag) -> Result<(), DomainError> {
        for resource in &tag.resource_ids {
            exists(&self.0.resources, resource)?;
        }
        self.0.tags.insert(tag.id, tag);
        Ok(())
    }
    fn archive(&mut self, target: ArchiveTarget, archived: bool) -> Result<(), DomainError> {
        let flag = match target {
            ArchiveTarget::Subject(id) => {
                self.0.subjects.get_mut(&id).map(|item| &mut item.archived)
            }
            ArchiveTarget::Goal(id) => self.0.goals.get_mut(&id).map(|item| &mut item.archived),
            ArchiveTarget::Collection(id) => self
                .0
                .collections
                .get_mut(&id)
                .map(|item| &mut item.archived),
            ArchiveTarget::Resource(id) => {
                self.0.resources.get_mut(&id).map(|item| &mut item.archived)
            }
            ArchiveTarget::Placement(id) => self
                .0
                .placements
                .get_mut(&id)
                .map(|item| &mut item.archived),
            ArchiveTarget::Unit(id) => self.0.units.get_mut(&id).map(|item| &mut item.archived),
            ArchiveTarget::Path(id) => self.0.paths.get_mut(&id).map(|item| &mut item.archived),
        }
        .ok_or(DomainError::NotFound)?;
        *flag = archived;
        Ok(())
    }
    pub(super) fn validate_units(&self) -> Result<(), DomainError> {
        for (id, unit) in &self.0.units {
            self.validate_unit(id, unit)?;
        }
        self.validate_paths()?;
        self.validate_relations()?;
        self.validate_tags()
    }
    fn validate_unit(&self, id: &UnitId, unit: &Unit) -> Result<(), DomainError> {
        super::validation::same(id, &unit.id)?;
        title(&unit.title)?;
        super::validation::text(&unit.objective)?;
        unique_refs(&unit.goal_ids, &self.0.goals)?;
        unique_refs(&unit.resource_ids, &self.0.resources)
    }
    fn validate_tags(&self) -> Result<(), DomainError> {
        for (id, tag) in &self.0.tags {
            super::validation::same(id, &tag.id)?;
            title(&tag.title)?;
            unique_refs(&tag.resource_ids, &self.0.resources)?;
        }
        Ok(())
    }
    fn validate_paths(&self) -> Result<(), DomainError> {
        let mut ids = BTreeSet::new();
        for (id, path) in &self.0.paths {
            super::validation::same(id, &path.id)?;
            title(&path.title)?;
            for step in &path.steps {
                exists(&self.0.units, &step.unit_id)?;
                if !ids.insert(step.id) {
                    return Err(DomainError::InvalidRequest);
                }
            }
        }
        Ok(())
    }
    fn validate_relations(&self) -> Result<(), DomainError> {
        let mut edges = BTreeSet::new();
        for (id, relation) in &self.0.relations {
            super::validation::same(id, &relation.id)?;
            exists(&self.0.units, &relation.from)?;
            exists(&self.0.units, &relation.to)?;
            let kind = match relation.kind {
                RelationKind::Prerequisite => 0,
                RelationKind::Related => 1,
                RelationKind::Reference => 2,
            };
            if !edges.insert((kind, relation.from, relation.to)) {
                return Err(DomainError::InvalidRequest);
            }
        }
        super::validation::acyclic(
            self.0.units.keys().copied(),
            self.0
                .relations
                .values()
                .filter(|r| r.kind == RelationKind::Prerequisite)
                .map(|r| (r.from, r.to)),
        )
    }
}
fn unique_refs<K: Copy + Ord, V>(ids: &[K], records: &BTreeMap<K, V>) -> Result<(), DomainError> {
    let mut seen = BTreeSet::new();
    for id in ids {
        exists(records, id)?;
        if !seen.insert(*id) {
            return Err(DomainError::InvalidRequest);
        }
    }
    Ok(())
}
