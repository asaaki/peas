use super::types::{PeaPriority, PeaStatus, PeaType};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pea {
    pub id: String,
    pub title: String,

    #[serde(rename = "type")]
    pub pea_type: PeaType,

    #[serde(default)]
    pub status: PeaStatus,

    #[serde(default)]
    pub priority: PeaPriority,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub blocking: Vec<String>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub assets: Vec<String>,

    #[serde(default)]
    pub created: DateTime<Utc>,

    #[serde(default)]
    pub updated: DateTime<Utc>,

    #[serde(skip)]
    pub body: String,
}

impl Pea {
    pub fn new(id: String, title: String, pea_type: PeaType) -> Self {
        let now = Utc::now();
        Self {
            id,
            title,
            pea_type,
            status: PeaStatus::default(),
            priority: PeaPriority::default(),
            tags: Vec::new(),
            parent: None,
            blocking: Vec::new(),
            assets: Vec::new(),
            created: now,
            updated: now,
            body: String::new(),
        }
    }

    pub fn with_status(mut self, status: PeaStatus) -> Self {
        self.status = status;
        self
    }

    pub fn with_priority(mut self, priority: PeaPriority) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    pub fn with_parent(mut self, parent: Option<String>) -> Self {
        self.parent = parent;
        self
    }

    pub fn with_blocking(mut self, blocking: Vec<String>) -> Self {
        self.blocking = blocking;
        self
    }

    pub fn with_body(mut self, body: String) -> Self {
        self.body = body;
        self
    }

    pub fn touch(&mut self) {
        self.updated = Utc::now();
    }

    pub fn is_open(&self) -> bool {
        matches!(
            self.status,
            PeaStatus::Draft | PeaStatus::Todo | PeaStatus::InProgress
        )
    }

    pub fn is_closed(&self) -> bool {
        matches!(self.status, PeaStatus::Completed | PeaStatus::Scrapped)
    }
}
