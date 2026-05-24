use serde::{Deserialize, Serialize};

/// Supported entity categories persisted in the knowledge store.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntityKind {
    /// Business or problem-space domain boundary.
    Domain,
    /// Multi-component system.
    System,
    /// Project or service unit.
    Project,
    /// Reusable code library.
    Library,
    /// Tag-only classification entity.
    Tag,
    /// Captured lesson note.
    Lesson,
    /// Captured issue note.
    Issue,
}

impl EntityKind {
    /// Returns the canonical lowercase representation used in SQLite rows.
    pub fn as_str(self) -> &'static str {
        match self {
            EntityKind::Domain => "domain",
            EntityKind::System => "system",
            EntityKind::Project => "project",
            EntityKind::Library => "library",
            EntityKind::Tag => "tag",
            EntityKind::Lesson => "lesson",
            EntityKind::Issue => "issue",
        }
    }
}

/// Supported typed relationships between entities.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelationshipKind {
    /// Hierarchical containment relation.
    Contains,
    /// Ownership relation.
    Owns,
    /// Publication relation (for packages/libraries).
    Publishes,
    /// Tag assignment relation.
    TaggedAs,
    /// Generic non-hierarchical relation.
    RelatedTo,
}

impl RelationshipKind {
    /// Returns the canonical lowercase representation used in SQLite rows.
    pub fn as_str(self) -> &'static str {
        match self {
            RelationshipKind::Contains => "contains",
            RelationshipKind::Owns => "owns",
            RelationshipKind::Publishes => "publishes",
            RelationshipKind::TaggedAs => "tagged_as",
            RelationshipKind::RelatedTo => "related_to",
        }
    }
}
