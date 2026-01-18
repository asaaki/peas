use crate::error::{PeasError, Result};
use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum PeaType {
    Milestone,
    Epic,
    Feature,
    Bug,
    #[default]
    Task,
}

impl fmt::Display for PeaType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PeaType::Milestone => write!(f, "milestone"),
            PeaType::Epic => write!(f, "epic"),
            PeaType::Feature => write!(f, "feature"),
            PeaType::Bug => write!(f, "bug"),
            PeaType::Task => write!(f, "task"),
        }
    }
}

impl FromStr for PeaType {
    type Err = PeasError;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "milestone" => Ok(PeaType::Milestone),
            "epic" => Ok(PeaType::Epic),
            "feature" => Ok(PeaType::Feature),
            "bug" => Ok(PeaType::Bug),
            "task" => Ok(PeaType::Task),
            _ => Err(PeasError::Parse(format!("Invalid pea type: {}", s))),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum PeaStatus {
    Draft,
    #[default]
    Todo,
    #[serde(rename = "in-progress")]
    InProgress,
    Completed,
    Scrapped,
}

impl fmt::Display for PeaStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PeaStatus::Draft => write!(f, "draft"),
            PeaStatus::Todo => write!(f, "todo"),
            PeaStatus::InProgress => write!(f, "in-progress"),
            PeaStatus::Completed => write!(f, "completed"),
            PeaStatus::Scrapped => write!(f, "scrapped"),
        }
    }
}

impl FromStr for PeaStatus {
    type Err = PeasError;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "draft" => Ok(PeaStatus::Draft),
            "todo" => Ok(PeaStatus::Todo),
            "in-progress" | "inprogress" | "in_progress" => Ok(PeaStatus::InProgress),
            "completed" | "done" => Ok(PeaStatus::Completed),
            "scrapped" | "cancelled" | "canceled" => Ok(PeaStatus::Scrapped),
            _ => Err(PeasError::Parse(format!("Invalid pea status: {}", s))),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum PeaPriority {
    Critical,
    High,
    #[default]
    Normal,
    Low,
    Deferred,
}

impl fmt::Display for PeaPriority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PeaPriority::Critical => write!(f, "critical"),
            PeaPriority::High => write!(f, "high"),
            PeaPriority::Normal => write!(f, "normal"),
            PeaPriority::Low => write!(f, "low"),
            PeaPriority::Deferred => write!(f, "deferred"),
        }
    }
}

impl FromStr for PeaPriority {
    type Err = PeasError;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "critical" | "p0" => Ok(PeaPriority::Critical),
            "high" | "p1" => Ok(PeaPriority::High),
            "normal" | "p2" => Ok(PeaPriority::Normal),
            "low" | "p3" => Ok(PeaPriority::Low),
            "deferred" | "p4" => Ok(PeaPriority::Deferred),
            _ => Err(PeasError::Parse(format!("Invalid priority: {}", s))),
        }
    }
}
