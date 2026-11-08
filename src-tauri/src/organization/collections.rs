use super::*;
use std::collections::BTreeSet;

impl Organization {
    pub(super) fn active_subject(&self, id: SubjectId) -> Result<(), DomainError> {
        let item = self.0.subjects.get(&id).ok_or(DomainError::NotFound)?;
        active(item.archived)
    }
    pub(super) fn active_collection(&self, id: CollectionId) -> Result<(), DomainError> {
        let mut current = Some(id);
        while let Some(id) = current {
            let item = self.0.collections.get(&id).ok_or(DomainError::NotFound)?;
            active(item.archived)?;
            self.active_subject(item.subject_id)?;
            current = item.parent_id;
        }
        Ok(())
    }
    fn destination(
        &self,
        subject: SubjectId,
        parent: Option<CollectionId>,
    ) -> Result<(), DomainError> {
        self.active_subject(subject)?;
        if let Some(id) = parent {
            self.active_collection(id)?;
            if self.0.collections[&id].subject_id != subject {
                return Err(DomainError::InvalidRequest);
            }
        }
        Ok(())
    }
    fn sibling_ids(&self, subject: SubjectId, parent: Option<CollectionId>) -> Vec<CollectionId> {
        let mut records: Vec<_> = self
            .0
            .collections
            .values()
            .filter(|c| c.subject_id == subject && c.parent_id == parent)
            .collect();
        records.sort_by_key(|c| c.position);
        records.into_iter().map(|c| c.id).collect()
    }
    pub(super) fn apply_collections(&mut self, command: Command) -> Result<(), DomainError> {
        match command {
            Command::CreateCollection {
                id,
                subject_id,
                parent_id,
                title,
            } => {
                self.destination(subject_id, parent_id)?;
                let position = self.sibling_ids(subject_id, parent_id).len() as u32;
                insert(
                    &mut self.0.collections,
                    id,
                    Collection {
                        id,
                        subject_id,
                        parent_id,
                        title,
                        position,
                        archived: false,
                    },
                )
            }
            Command::MoveCollection {
                id,
                subject_id,
                parent_id,
            } => self.move_collection(id, subject_id, parent_id),
            Command::ReorderCollections {
                subject_id,
                parent_id,
                ordered_ids,
            } => {
                self.destination(subject_id, parent_id)?;
                permutation(&self.sibling_ids(subject_id, parent_id), &ordered_ids)?;
                order_records(&mut self.0.collections, &ordered_ids, |collection| {
                    &mut collection.position
                });
                Ok(())
            }
            other => self.apply_resources(other),
        }
    }
    fn move_collection(
        &mut self,
        id: CollectionId,
        subject: SubjectId,
        parent: Option<CollectionId>,
    ) -> Result<(), DomainError> {
        self.active_collection(id)?;
        self.destination(subject, parent)?;
        let subtree = self.subtree(id);
        if parent.is_some_and(|p| subtree.contains(&p)) {
            return Err(DomainError::Cycle);
        }
        let old = self.0.collections[&id].clone();
        let old_siblings: Vec<_> = self
            .sibling_ids(old.subject_id, old.parent_id)
            .into_iter()
            .filter(|other| *other != id)
            .collect();
        let mut destination: Vec<_> = self
            .sibling_ids(subject, parent)
            .into_iter()
            .filter(|other| *other != id)
            .collect();
        destination.push(id);
        for child in subtree {
            self.0
                .collections
                .get_mut(&child)
                .ok_or(DomainError::NotFound)?
                .subject_id = subject;
        }
        self.0
            .collections
            .get_mut(&id)
            .ok_or(DomainError::NotFound)?
            .parent_id = parent;
        order_records(&mut self.0.collections, &old_siblings, |collection| {
            &mut collection.position
        });
        order_records(&mut self.0.collections, &destination, |collection| {
            &mut collection.position
        });
        Ok(())
    }
    fn subtree(&self, root: CollectionId) -> BTreeSet<CollectionId> {
        let mut children: BTreeMap<CollectionId, Vec<CollectionId>> = BTreeMap::new();
        for collection in self.0.collections.values() {
            if let Some(parent) = collection.parent_id {
                children.entry(parent).or_default().push(collection.id);
            }
        }
        let mut result = BTreeSet::new();
        let mut pending = vec![root];
        while let Some(id) = pending.pop() {
            if result.insert(id) {
                pending.extend(children.get(&id).into_iter().flatten().copied());
            }
        }
        result
    }
    pub(super) fn apply_resources(&mut self, command: Command) -> Result<(), DomainError> {
        match command {
            Command::CreateResource { id, title } => insert(
                &mut self.0.resources,
                id,
                Resource {
                    id,
                    title,
                    archived: false,
                },
            ),
            Command::RenameResource { id, title } => {
                let resource = self.0.resources.get_mut(&id).ok_or(DomainError::NotFound)?;
                active(resource.archived)?;
                resource.title = title;
                Ok(())
            }
            Command::Place {
                id,
                collection_id,
                target,
            } => self.place(id, collection_id, target),
            Command::MovePlacement { id, collection_id } => self.move_placement(id, collection_id),
            Command::ReorderPlacements {
                collection_id,
                ordered_ids,
            } => self.reorder_placements(collection_id, &ordered_ids),
            other => self.apply_units(other),
        }
    }
    fn place(
        &mut self,
        id: PlacementId,
        collection_id: CollectionId,
        target: Target,
    ) -> Result<(), DomainError> {
        self.active_collection(collection_id)?;
        self.active_target(target)?;
        let position = self.placement_ids(collection_id).len() as u32;
        insert(
            &mut self.0.placements,
            id,
            Placement {
                id,
                collection_id,
                target,
                position,
                archived: false,
            },
        )
    }
    fn reorder_placements(
        &mut self,
        collection_id: CollectionId,
        ordered_ids: &[PlacementId],
    ) -> Result<(), DomainError> {
        self.active_collection(collection_id)?;
        permutation(&self.placement_ids(collection_id), ordered_ids)?;
        order_records(&mut self.0.placements, ordered_ids, |placement| {
            &mut placement.position
        });
        Ok(())
    }
    fn active_target(&self, target: Target) -> Result<(), DomainError> {
        let archived = match target {
            Target::Resource(id) => {
                self.0
                    .resources
                    .get(&id)
                    .ok_or(DomainError::NotFound)?
                    .archived
            }
            Target::Unit(id) => self.0.units.get(&id).ok_or(DomainError::NotFound)?.archived,
        };
        active(archived)
    }
    fn placement_ids(&self, collection: CollectionId) -> Vec<PlacementId> {
        let mut records: Vec<_> = self
            .0
            .placements
            .values()
            .filter(|p| p.collection_id == collection)
            .collect();
        records.sort_by_key(|p| p.position);
        records.into_iter().map(|p| p.id).collect()
    }
    fn move_placement(
        &mut self,
        id: PlacementId,
        collection: CollectionId,
    ) -> Result<(), DomainError> {
        self.active_collection(collection)?;
        let old = self
            .0
            .placements
            .get(&id)
            .ok_or(DomainError::NotFound)?
            .clone();
        active(old.archived)?;
        self.active_collection(old.collection_id)?;
        self.active_target(old.target)?;
        let prior: Vec<_> = self
            .placement_ids(old.collection_id)
            .into_iter()
            .filter(|other| *other != id)
            .collect();
        let mut target: Vec<_> = self
            .placement_ids(collection)
            .into_iter()
            .filter(|other| *other != id)
            .collect();
        target.push(id);
        self.0
            .placements
            .get_mut(&id)
            .ok_or(DomainError::NotFound)?
            .collection_id = collection;
        order_records(&mut self.0.placements, &prior, |placement| {
            &mut placement.position
        });
        order_records(&mut self.0.placements, &target, |placement| {
            &mut placement.position
        });
        Ok(())
    }
}
pub(super) fn active(archived: bool) -> Result<(), DomainError> {
    if archived {
        Err(DomainError::InvalidRequest)
    } else {
        Ok(())
    }
}
pub(super) fn permutation<T: Ord + Copy>(
    existing: &[T],
    proposed: &[T],
) -> Result<(), DomainError> {
    let supplied: BTreeSet<_> = proposed.iter().copied().collect();
    let actual: BTreeSet<_> = existing.iter().copied().collect();
    if supplied.len() != proposed.len() || supplied != actual {
        Err(DomainError::InvalidRequest)
    } else {
        Ok(())
    }
}

fn order_records<K: Ord, V>(
    records: &mut BTreeMap<K, V>,
    ids: &[K],
    position: impl Fn(&mut V) -> &mut u32,
) {
    for (index, id) in ids.iter().enumerate() {
        if let Some(record) = records.get_mut(id) {
            *position(record) = index as u32;
        }
    }
}
