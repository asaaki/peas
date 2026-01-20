use crate::model::{self, Memory as ModelMemory, Pea as ModelPea};
use async_graphql::{Enum, InputObject, SimpleObject};

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub enum PeaType {
    Milestone,
    Epic,
    Story,
    Feature,
    Bug,
    Chore,
    Research,
    Task,
}

impl From<model::PeaType> for PeaType {
    fn from(t: model::PeaType) -> Self {
        match t {
            model::PeaType::Milestone => PeaType::Milestone,
            model::PeaType::Epic => PeaType::Epic,
            model::PeaType::Story => PeaType::Story,
            model::PeaType::Feature => PeaType::Feature,
            model::PeaType::Bug => PeaType::Bug,
            model::PeaType::Chore => PeaType::Chore,
            model::PeaType::Research => PeaType::Research,
            model::PeaType::Task => PeaType::Task,
        }
    }
}

impl From<PeaType> for model::PeaType {
    fn from(t: PeaType) -> Self {
        match t {
            PeaType::Milestone => model::PeaType::Milestone,
            PeaType::Epic => model::PeaType::Epic,
            PeaType::Story => model::PeaType::Story,
            PeaType::Feature => model::PeaType::Feature,
            PeaType::Bug => model::PeaType::Bug,
            PeaType::Chore => model::PeaType::Chore,
            PeaType::Research => model::PeaType::Research,
            PeaType::Task => model::PeaType::Task,
        }
    }
}

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub enum PeaStatus {
    Draft,
    Todo,
    InProgress,
    Completed,
    Scrapped,
}

impl From<model::PeaStatus> for PeaStatus {
    fn from(s: model::PeaStatus) -> Self {
        match s {
            model::PeaStatus::Draft => PeaStatus::Draft,
            model::PeaStatus::Todo => PeaStatus::Todo,
            model::PeaStatus::InProgress => PeaStatus::InProgress,
            model::PeaStatus::Completed => PeaStatus::Completed,
            model::PeaStatus::Scrapped => PeaStatus::Scrapped,
        }
    }
}

impl From<PeaStatus> for model::PeaStatus {
    fn from(s: PeaStatus) -> Self {
        match s {
            PeaStatus::Draft => model::PeaStatus::Draft,
            PeaStatus::Todo => model::PeaStatus::Todo,
            PeaStatus::InProgress => model::PeaStatus::InProgress,
            PeaStatus::Completed => model::PeaStatus::Completed,
            PeaStatus::Scrapped => model::PeaStatus::Scrapped,
        }
    }
}

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub enum PeaPriority {
    Critical,
    High,
    Normal,
    Low,
    Deferred,
}

impl From<model::PeaPriority> for PeaPriority {
    fn from(p: model::PeaPriority) -> Self {
        match p {
            model::PeaPriority::Critical => PeaPriority::Critical,
            model::PeaPriority::High => PeaPriority::High,
            model::PeaPriority::Normal => PeaPriority::Normal,
            model::PeaPriority::Low => PeaPriority::Low,
            model::PeaPriority::Deferred => PeaPriority::Deferred,
        }
    }
}

impl From<PeaPriority> for model::PeaPriority {
    fn from(p: PeaPriority) -> Self {
        match p {
            PeaPriority::Critical => model::PeaPriority::Critical,
            PeaPriority::High => model::PeaPriority::High,
            PeaPriority::Normal => model::PeaPriority::Normal,
            PeaPriority::Low => model::PeaPriority::Low,
            PeaPriority::Deferred => model::PeaPriority::Deferred,
        }
    }
}

#[derive(SimpleObject)]
pub struct Pea {
    pub id: String,
    pub title: String,
    pub pea_type: PeaType,
    pub status: PeaStatus,
    pub priority: PeaPriority,
    pub tags: Vec<String>,
    pub parent: Option<String>,
    pub blocking: Vec<String>,
    pub created: String,
    pub updated: String,
    pub body: String,
}

impl From<ModelPea> for Pea {
    fn from(p: ModelPea) -> Self {
        Self {
            id: p.id,
            title: p.title,
            pea_type: p.pea_type.into(),
            status: p.status.into(),
            priority: p.priority.into(),
            tags: p.tags,
            parent: p.parent,
            blocking: p.blocking,
            created: p.created.to_rfc3339(),
            updated: p.updated.to_rfc3339(),
            body: p.body,
        }
    }
}

#[derive(InputObject)]
pub struct PeaFilter {
    pub pea_type: Option<PeaType>,
    pub status: Option<PeaStatus>,
    pub priority: Option<PeaPriority>,
    pub parent: Option<String>,
    pub tag: Option<String>,
    pub is_open: Option<bool>,
}

#[derive(InputObject)]
pub struct CreatePeaInput {
    pub title: String,
    pub pea_type: Option<PeaType>,
    pub status: Option<PeaStatus>,
    pub priority: Option<PeaPriority>,
    pub body: Option<String>,
    pub parent: Option<String>,
    pub blocking: Option<Vec<String>>,
    pub tags: Option<Vec<String>>,
}

#[derive(InputObject)]
pub struct UpdatePeaInput {
    pub id: String,
    pub title: Option<String>,
    pub pea_type: Option<PeaType>,
    pub status: Option<PeaStatus>,
    pub priority: Option<PeaPriority>,
    pub body: Option<String>,
    pub parent: Option<String>,
    pub blocking: Option<Vec<String>>,
    pub add_tags: Option<Vec<String>>,
    pub remove_tags: Option<Vec<String>>,
}

#[derive(SimpleObject)]
pub struct PeaConnection {
    pub nodes: Vec<Pea>,
    pub total_count: usize,
}

#[derive(SimpleObject)]
pub struct ProjectStats {
    pub total: usize,
    pub by_status: StatusCounts,
    pub by_type: TypeCounts,
}

#[derive(SimpleObject)]
pub struct StatusCounts {
    pub draft: usize,
    pub todo: usize,
    pub in_progress: usize,
    pub completed: usize,
    pub scrapped: usize,
}

#[derive(SimpleObject)]
pub struct TypeCounts {
    pub milestone: usize,
    pub epic: usize,
    pub story: usize,
    pub feature: usize,
    pub bug: usize,
    pub chore: usize,
    pub research: usize,
    pub task: usize,
}

#[derive(SimpleObject, Clone)]
pub struct Memory {
    pub key: String,
    pub content: String,
    pub tags: Vec<String>,
    #[graphql(name = "created")]
    pub created: String,
    #[graphql(name = "updated")]
    pub updated: String,
}

impl From<ModelMemory> for Memory {
    fn from(m: ModelMemory) -> Self {
        Memory {
            key: m.key,
            content: m.content,
            tags: m.tags,
            created: m.created.to_rfc3339(),
            updated: m.updated.to_rfc3339(),
        }
    }
}

#[derive(InputObject)]
pub struct CreateMemoryInput {
    pub key: String,
    pub content: String,
    pub tags: Option<Vec<String>>,
}

#[derive(InputObject)]
pub struct UpdateMemoryInput {
    pub key: String,
    pub content: String,
    pub tags: Option<Vec<String>>,
}
