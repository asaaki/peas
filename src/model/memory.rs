use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memory {
    pub key: String,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,

    #[serde(default)]
    pub created: DateTime<Utc>,

    #[serde(default)]
    pub updated: DateTime<Utc>,

    #[serde(skip)]
    pub content: String,
}

impl Memory {
    pub fn new(key: String) -> Self {
        let now = Utc::now();
        Self {
            key,
            tags: Vec::new(),
            created: now,
            updated: now,
            content: String::new(),
        }
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    pub fn with_content(mut self, content: String) -> Self {
        self.content = content;
        self
    }

    pub fn touch(&mut self) {
        self.updated = Utc::now();
    }
}
