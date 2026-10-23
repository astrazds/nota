use uuid::Uuid;

use crate::model::Note;

/// A note identity that appears twice in one notebook.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DuplicateNoteId(pub Uuid);

/// Active notes and Recently Deleted. Identities are unique in each list and disjoint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Notebook {
    active: Vec<Note>,
    deleted: Vec<Note>,
}

impl Notebook {
    pub fn try_new(active: Vec<Note>, deleted: Vec<Note>) -> Result<Self, DuplicateNoteId> {
        let mut seen = std::collections::HashSet::with_capacity(active.len() + deleted.len());
        for note in active.iter().chain(&deleted) {
            if !seen.insert(note.id) {
                return Err(DuplicateNoteId(note.id));
            }
        }
        Ok(Self { active, deleted })
    }

    pub fn empty() -> Self {
        Self::try_new(Vec::new(), Vec::new())
            .expect("an empty notebook has no duplicate identities")
    }

    pub fn active(&self) -> &[Note] {
        &self.active
    }

    pub fn deleted(&self) -> &[Note] {
        &self.deleted
    }

    pub fn parts(&self) -> (&[Note], &[Note]) {
        (&self.active, &self.deleted)
    }

    pub(crate) fn active_mut(&mut self) -> &mut Vec<Note> {
        &mut self.active
    }

    pub(crate) fn deleted_mut(&mut self) -> &mut Vec<Note> {
        &mut self.deleted
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_identity_cannot_appear_twice() {
        let note = Note::new("Twice".to_string(), String::new());
        let error = Notebook::try_new(vec![note.clone()], vec![note.clone()]).unwrap_err();
        assert_eq!(error, DuplicateNoteId(note.id));
        let error = Notebook::try_new(vec![note.clone(), note.clone()], Vec::new()).unwrap_err();
        assert_eq!(error, DuplicateNoteId(note.id));
    }
}
