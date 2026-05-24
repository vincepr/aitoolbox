use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntityKind {
    Domain,
    System,
    Project,
    Library,
    Tag,
    Lesson,
    Issue,
}

impl EntityKind {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelationshipKind {
    Contains,
    Owns,
    Publishes,
    TaggedAs,
    RelatedTo,
}

impl RelationshipKind {
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
