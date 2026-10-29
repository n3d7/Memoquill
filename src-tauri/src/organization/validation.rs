use super::*;
use std::collections::{BTreeSet, VecDeque};

impl Organization {
    pub(super) fn validate(&self) -> Result<(), DomainError> {
        if self.0.workspace_id.is_nil() || self.0.revision > MAX_REVISION {
            return Err(DomainError::InvalidRequest);
        }
        self.validate_subjects()?;
        self.validate_collections()?;
        self.validate_resources()?;
        self.validate_units()
    }
    fn validate_subjects(&self) -> Result<(), DomainError> {
        for (id, subject) in &self.0.subjects {
            same(id, &subject.id)?;
            title(&subject.title)?;
            if let Some(note) = &subject.prior_knowledge {
                text(note)?;
            }
        }
        self.validate_goals()
    }
    fn validate_goals(&self) -> Result<(), DomainError> {
        for (id, goal) in &self.0.goals {
            same(id, &goal.id)?;
            title(&goal.title)?;
            text(&goal.criteria)?;
            exists(&self.0.subjects, &goal.subject_id)?;
        }
        Ok(())
    }
    fn validate_collections(&self) -> Result<(), DomainError> {
        let mut groups: BTreeMap<_, Vec<u32>> = BTreeMap::new();
        for (id, collection) in &self.0.collections {
            same(id, &collection.id)?;
            title(&collection.title)?;
            exists(&self.0.subjects, &collection.subject_id)?;
            self.validate_parent(collection)?;
            groups
                .entry((collection.subject_id, collection.parent_id))
                .or_default()
                .push(collection.position);
        }
        dense_orders(groups.into_values())?;
        acyclic(
            self.0.collections.keys().copied(),
            self.0
                .collections
                .values()
                .filter_map(|c| c.parent_id.map(|p| (c.id, p))),
        )
    }
    fn validate_parent(&self, collection: &Collection) -> Result<(), DomainError> {
        if let Some(parent) = collection.parent_id {
            let other = self
                .0
                .collections
                .get(&parent)
                .ok_or(DomainError::NotFound)?;
            same(&collection.subject_id, &other.subject_id)?;
        }
        Ok(())
    }
    fn validate_resources(&self) -> Result<(), DomainError> {
        for (id, resource) in &self.0.resources {
            same(id, &resource.id)?;
            title(&resource.title)?;
        }
        self.validate_placements()
    }
    fn validate_placements(&self) -> Result<(), DomainError> {
        let mut groups: BTreeMap<_, Vec<u32>> = BTreeMap::new();
        for (id, placement) in &self.0.placements {
            same(id, &placement.id)?;
            exists(&self.0.collections, &placement.collection_id)?;
            match placement.target {
                Target::Resource(id) => exists(&self.0.resources, &id)?,
                Target::Unit(id) => exists(&self.0.units, &id)?,
            }
            groups
                .entry(placement.collection_id)
                .or_default()
                .push(placement.position);
        }
        dense_orders(groups.into_values())
    }
}
pub(super) fn exists<K: Ord, V>(items: &BTreeMap<K, V>, id: &K) -> Result<(), DomainError> {
    if items.contains_key(id) {
        Ok(())
    } else {
        Err(DomainError::NotFound)
    }
}
pub(super) fn same<T: PartialEq>(a: &T, b: &T) -> Result<(), DomainError> {
    if a == b {
        Ok(())
    } else {
        Err(DomainError::InvalidRequest)
    }
}
pub(super) fn text(value: &str) -> Result<(), DomainError> {
    if value.len() > 65_536 || value.contains('\0') {
        Err(DomainError::InvalidRequest)
    } else {
        Ok(())
    }
}
fn dense_orders(groups: impl Iterator<Item = Vec<u32>>) -> Result<(), DomainError> {
    for mut positions in groups {
        positions.sort_unstable();
        if positions.iter().enumerate().any(|(i, p)| *p as usize != i) {
            return Err(DomainError::InvalidRequest);
        }
    }
    Ok(())
}
pub(super) fn acyclic<K: Copy + Ord>(
    nodes: impl Iterator<Item = K>,
    edges: impl Iterator<Item = (K, K)>,
) -> Result<(), DomainError> {
    let mut indegree: BTreeMap<K, usize> = nodes.map(|id| (id, 0)).collect();
    let mut children: BTreeMap<K, BTreeSet<K>> = BTreeMap::new();
    for (from, to) in edges {
        if children.entry(from).or_default().insert(to) {
            *indegree.get_mut(&to).ok_or(DomainError::NotFound)? += 1;
        }
    }
    let mut pending: VecDeque<_> = indegree
        .iter()
        .filter(|(_, n)| **n == 0)
        .map(|(id, _)| *id)
        .collect();
    let mut visited = 0;
    while let Some(id) = pending.pop_front() {
        visited += 1;
        for child in children.get(&id).into_iter().flatten() {
            let count = indegree.get_mut(child).ok_or(DomainError::NotFound)?;
            *count -= 1;
            if *count == 0 {
                pending.push_back(*child);
            }
        }
    }
    if visited == indegree.len() {
        Ok(())
    } else {
        Err(DomainError::Cycle)
    }
}
