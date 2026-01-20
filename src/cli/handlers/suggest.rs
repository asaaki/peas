use crate::model::{PeaPriority, PeaStatus, PeaType};
use anyhow::Result;
use colored::Colorize;
use std::collections::HashMap;

use super::CommandContext;
use super::utils::print_pea;

pub fn handle_suggest(ctx: &CommandContext, json: bool, limit: usize) -> Result<()> {
    let peas = ctx.repo.list()?;

    // Build a map of ticket ID to completion status
    let status_map: HashMap<String, PeaStatus> =
        peas.iter().map(|p| (p.id.clone(), p.status)).collect();

    // Calculate how many tickets each ticket is blocking
    let mut blocking_count: HashMap<String, usize> = HashMap::new();
    for pea in &peas {
        for blocked_id in &pea.blocking {
            *blocking_count.entry(blocked_id.clone()).or_insert(0) += 1;
        }
    }

    // Filter to open, actionable items (not milestones/epics which are containers)
    // Also filter out tickets with unmet dependencies
    let mut candidates: Vec<_> = peas
        .iter()
        .filter(|p| {
            if !p.is_open() || matches!(p.pea_type, PeaType::Milestone | PeaType::Epic) {
                return false;
            }

            // Check if all blocking dependencies are completed
            for blocker_id in &p.blocking {
                if let Some(status) = status_map.get(blocker_id) {
                    if *status != PeaStatus::Completed && *status != PeaStatus::Scrapped {
                        return false; // Has unmet dependency
                    }
                }
            }

            true
        })
        .collect();

    if candidates.is_empty() {
        // No regular tickets found, check for epics/milestones without actionable children
        let epics_and_milestones: Vec<_> = peas
            .iter()
            .filter(|p| p.is_open() && (matches!(p.pea_type, PeaType::Epic | PeaType::Milestone)))
            .collect();

        if !epics_and_milestones.is_empty() {
            // Check if any have open children
            let parent_has_open_children = |parent_id: &str| -> bool {
                peas.iter().any(|p| {
                    p.parent.as_deref() == Some(parent_id)
                        && p.is_open()
                        && !matches!(p.pea_type, PeaType::Epic | PeaType::Milestone)
                })
            };

            // Filter to epics/milestones with no open children
            let empty_epics: Vec<_> = epics_and_milestones
                .iter()
                .filter(|e| !parent_has_open_children(&e.id))
                .collect();

            if !empty_epics.is_empty() {
                let epic = empty_epics[0];
                if json {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&serde_json::json!({
                            "suggestion": epic,
                            "reason": "No actionable tickets - consider breaking down this epic or marking it complete",
                            "type": "epic_without_children"
                        }))?
                    );
                } else {
                    println!("{}: No actionable tickets found", "Note".yellow().bold());
                    println!();
                    println!(
                        "Found open {} with no actionable children:",
                        if epic.pea_type == PeaType::Epic {
                            "epic"
                        } else {
                            "milestone"
                        }
                    );
                    println!();
                    print_pea(epic);
                    println!();
                    println!(
                        "{}: Consider breaking this down into tickets or marking it complete.",
                        "Suggestion".green().bold()
                    );
                }
                return Ok(());
            }
        }

        // Truly nothing to suggest
        if json {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "suggestion": null,
                    "reason": "No open actionable tickets found (some may be blocked)"
                }))?
            );
        } else {
            println!("No open actionable tickets found (some may be blocked by dependencies).");
        }
        return Ok(());
    }

    // Sort by: in-progress first, then blocking count, then priority, then by type
    candidates.sort_by(|a, b| {
        // In-progress items first
        let a_in_progress = a.status == PeaStatus::InProgress;
        let b_in_progress = b.status == PeaStatus::InProgress;
        if a_in_progress != b_in_progress {
            return b_in_progress.cmp(&a_in_progress);
        }

        // Then by blocking count (tickets blocking more items come first)
        let a_blocks = blocking_count.get(&a.id).unwrap_or(&0);
        let b_blocks = blocking_count.get(&b.id).unwrap_or(&0);
        if a_blocks != b_blocks {
            return b_blocks.cmp(a_blocks);
        }

        // Then by priority
        let priority_order = |p: &PeaPriority| match p {
            PeaPriority::Critical => 0,
            PeaPriority::High => 1,
            PeaPriority::Normal => 2,
            PeaPriority::Low => 3,
            PeaPriority::Deferred => 4,
        };
        let a_pri = priority_order(&a.priority);
        let b_pri = priority_order(&b.priority);
        if a_pri != b_pri {
            return a_pri.cmp(&b_pri);
        }

        // Then by type (bugs before features before tasks)
        let type_order = |t: &PeaType| match t {
            PeaType::Bug => 0,
            PeaType::Feature => 1,
            PeaType::Story => 2,
            PeaType::Chore => 3,
            PeaType::Research => 4,
            PeaType::Task => 5,
            _ => 6,
        };
        type_order(&a.pea_type).cmp(&type_order(&b.pea_type))
    });

    // Take top N suggestions
    let num_suggestions = limit.min(candidates.len());
    let suggestions: Vec<_> = candidates.iter().take(num_suggestions).collect();

    if json {
        let suggestions_with_reasons: Vec<_> = suggestions
            .iter()
            .map(|s| {
                let blocks_count = blocking_count.get(&s.id).unwrap_or(&0);
                let reason = if s.status == PeaStatus::InProgress {
                    "Currently in progress".to_string()
                } else if *blocks_count > 0 {
                    format!("Blocking {} ticket(s)", blocks_count)
                } else if s.priority == PeaPriority::Critical {
                    "Critical priority".to_string()
                } else if s.priority == PeaPriority::High {
                    "High priority".to_string()
                } else if s.pea_type == PeaType::Bug {
                    "Bug fix".to_string()
                } else {
                    "Next in queue".to_string()
                };

                serde_json::json!({
                    "pea": s,
                    "reason": reason,
                    "blocks_count": blocks_count
                })
            })
            .collect();

        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "suggestions": suggestions_with_reasons,
                "count": num_suggestions
            }))?
        );
    } else {
        if num_suggestions == 1 {
            let suggestion = suggestions[0];
            let blocks_count = blocking_count.get(&suggestion.id).unwrap_or(&0);
            let reason = if suggestion.status == PeaStatus::InProgress {
                "Currently in progress".to_string()
            } else if *blocks_count > 0 {
                format!("Blocking {} ticket(s)", blocks_count)
            } else if suggestion.priority == PeaPriority::Critical {
                "Critical priority".to_string()
            } else if suggestion.priority == PeaPriority::High {
                "High priority".to_string()
            } else if suggestion.pea_type == PeaType::Bug {
                "Bug fix".to_string()
            } else {
                "Next in queue".to_string()
            };

            println!("{}: {}", "Suggested".green().bold(), reason);
            println!();
            print_pea(suggestion);
        } else {
            println!("{} {} suggestions:", "Top".green().bold(), num_suggestions);
            println!();
            for (i, suggestion) in suggestions.iter().enumerate() {
                let blocks_count = blocking_count.get(&suggestion.id).unwrap_or(&0);
                let reason = if suggestion.status == PeaStatus::InProgress {
                    "Currently in progress".to_string()
                } else if *blocks_count > 0 {
                    format!("Blocking {} ticket(s)", blocks_count)
                } else if suggestion.priority == PeaPriority::Critical {
                    "Critical priority".to_string()
                } else if suggestion.priority == PeaPriority::High {
                    "High priority".to_string()
                } else if suggestion.pea_type == PeaType::Bug {
                    "Bug fix".to_string()
                } else {
                    "Next in queue".to_string()
                };

                println!("{}. {} - {}", i + 1, reason.cyan(), suggestion.title);
                println!(
                    "   {} [{}] {}",
                    suggestion.id.dimmed(),
                    suggestion.pea_type,
                    suggestion.priority
                );
                if *blocks_count > 0 {
                    println!("   {} Unblocks {} ticket(s)", "⚠".yellow(), blocks_count);
                }
                println!();
            }
        }
    }

    Ok(())
}
