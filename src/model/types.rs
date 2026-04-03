use crate::error::{PeasError, Result};
use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};

/// The type of a pea (issue/ticket).
///
/// ```
/// use std::str::FromStr;
/// use peas::model::PeaType;
///
/// let t: PeaType = "bug".parse().unwrap();
/// assert_eq!(t, PeaType::Bug);
/// assert_eq!(t.to_string(), "bug");
///
/// // "spike" is an alias for Research
/// let r: PeaType = "spike".parse().unwrap();
/// assert_eq!(r, PeaType::Research);
///
/// // Parsing is case-insensitive
/// assert_eq!("BUG".parse::<PeaType>().unwrap(), PeaType::Bug);
///
/// // Invalid types return an error
/// assert!("invalid".parse::<PeaType>().is_err());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum PeaType {
    Milestone,
    Epic,
    Story,
    Feature,
    Bug,
    Chore,
    Research,
    #[default]
    Task,
}

impl fmt::Display for PeaType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PeaType::Milestone => write!(f, "milestone"),
            PeaType::Epic => write!(f, "epic"),
            PeaType::Story => write!(f, "story"),
            PeaType::Feature => write!(f, "feature"),
            PeaType::Bug => write!(f, "bug"),
            PeaType::Chore => write!(f, "chore"),
            PeaType::Research => write!(f, "research"),
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
            "story" => Ok(PeaType::Story),
            "feature" => Ok(PeaType::Feature),
            "bug" => Ok(PeaType::Bug),
            "chore" => Ok(PeaType::Chore),
            "research" | "spike" => Ok(PeaType::Research),
            "task" => Ok(PeaType::Task),
            _ => Err(PeasError::Parse(format!("Invalid pea type: {}", s))),
        }
    }
}

/// The status of a pea.
///
/// ```
/// use std::str::FromStr;
/// use peas::model::PeaStatus;
///
/// // Multiple aliases are supported
/// assert_eq!("in-progress".parse::<PeaStatus>().unwrap(), PeaStatus::InProgress);
/// assert_eq!("in_progress".parse::<PeaStatus>().unwrap(), PeaStatus::InProgress);
/// assert_eq!("inprogress".parse::<PeaStatus>().unwrap(), PeaStatus::InProgress);
///
/// // "done" and "cancelled" are aliases
/// assert_eq!("done".parse::<PeaStatus>().unwrap(), PeaStatus::Completed);
/// assert_eq!("cancelled".parse::<PeaStatus>().unwrap(), PeaStatus::Scrapped);
/// assert_eq!("canceled".parse::<PeaStatus>().unwrap(), PeaStatus::Scrapped);
/// ```
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

/// The priority of a pea.
///
/// ```
/// use std::str::FromStr;
/// use peas::model::PeaPriority;
///
/// // Short forms are supported
/// assert_eq!("p0".parse::<PeaPriority>().unwrap(), PeaPriority::Critical);
/// assert_eq!("p1".parse::<PeaPriority>().unwrap(), PeaPriority::High);
/// assert_eq!("p2".parse::<PeaPriority>().unwrap(), PeaPriority::Normal);
/// assert_eq!("p3".parse::<PeaPriority>().unwrap(), PeaPriority::Low);
/// assert_eq!("p4".parse::<PeaPriority>().unwrap(), PeaPriority::Deferred);
///
/// // Full names also work
/// assert_eq!("critical".parse::<PeaPriority>().unwrap(), PeaPriority::Critical);
/// ```
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
