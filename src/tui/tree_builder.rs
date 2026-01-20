use crate::model::{Pea, PeaStatus, PeaType};
use std::collections::{HashMap, HashSet};

/// A node in the tree view representing a pea and its depth
#[derive(Debug, Clone)]
pub struct TreeNode {
    pub pea: Pea,
    pub depth: usize,
    pub is_last: bool,           // Is this the last child at this level?
    pub parent_lines: Vec<bool>, // Which parent levels need continuing lines
}

/// Build a hierarchical tree structure from a flat list of peas
pub fn build_tree(filtered_peas: &[Pea]) -> Vec<TreeNode> {
    let mut tree_nodes = Vec::new();

    // Build a set of IDs that exist in filtered_peas for quick lookup
    let filtered_ids: HashSet<String> = filtered_peas.iter().map(|p| p.id.clone()).collect();

    // Build a map of parent -> children
    let mut children_map: HashMap<Option<String>, Vec<&Pea>> = HashMap::new();
    for pea in filtered_peas {
        // If the pea has a parent but that parent is not in the filtered set,
        // treat it as a root item (orphaned)
        let effective_parent = if let Some(ref parent_id) = pea.parent {
            if filtered_ids.contains(parent_id) {
                pea.parent.clone()
            } else {
                None // Parent not in filtered set, show as root
            }
        } else {
            None
        };

        children_map.entry(effective_parent).or_default().push(pea);
    }

    // Sort children by status (in-progress first, then todo, then completed) then by type hierarchy
    for children in children_map.values_mut() {
        children.sort_by(|a, b| {
            status_order(&a.status)
                .cmp(&status_order(&b.status))
                .then_with(|| type_order(&a.pea_type).cmp(&type_order(&b.pea_type)))
                .then_with(|| a.title.cmp(&b.title))
        });
    }

    // Start with root nodes (no parent or orphaned items)
    add_children(None, 0, Vec::new(), &children_map, &mut tree_nodes);

    tree_nodes
}

fn status_order(status: &PeaStatus) -> u8 {
    match status {
        PeaStatus::InProgress => 0,
        PeaStatus::Todo => 1,
        PeaStatus::Draft => 2,
        PeaStatus::Completed => 3,
        PeaStatus::Scrapped => 4,
    }
}

fn type_order(pea_type: &PeaType) -> u8 {
    match pea_type {
        PeaType::Milestone => 0,
        PeaType::Epic => 1,
        PeaType::Story => 2,
        PeaType::Feature => 3,
        PeaType::Bug => 4,
        PeaType::Chore => 5,
        PeaType::Research => 6,
        PeaType::Task => 7,
    }
}

/// Recursively build tree nodes
fn add_children(
    parent_id: Option<String>,
    depth: usize,
    parent_lines: Vec<bool>,
    children_map: &HashMap<Option<String>, Vec<&Pea>>,
    nodes: &mut Vec<TreeNode>,
) {
    if let Some(children) = children_map.get(&parent_id) {
        let count = children.len();
        for (i, pea) in children.iter().enumerate() {
            let is_last = i == count - 1;
            let mut current_parent_lines = parent_lines.clone();

            nodes.push(TreeNode {
                pea: (*pea).clone(),
                depth,
                is_last,
                parent_lines: current_parent_lines.clone(),
            });

            // For children, add whether this level continues
            // But only track continuation lines for depth > 0 (not for root items)
            if depth > 0 {
                current_parent_lines.push(!is_last);
            }
            add_children(
                Some(pea.id.clone()),
                depth + 1,
                current_parent_lines,
                children_map,
                nodes,
            );
        }
    }
}

/// Layer 2: Page table entry with references to tree nodes
#[derive(Debug, Clone)]
pub struct PageInfo {
    pub start_index: usize, // Starting index in tree_nodes for regular items
    pub item_count: usize,  // Number of actual items on this page
    pub parent_indices: Vec<usize>, // Indices of parent context nodes to show (top-down order)
}

/// Build a virtual page table that accounts for parent context rows
pub fn build_page_table(tree_nodes: &[TreeNode], page_height: usize) -> Vec<PageInfo> {
    let mut page_table = Vec::new();

    if tree_nodes.is_empty() || page_height == 0 {
        return page_table;
    }

    let mut current_index = 0;
    while current_index < tree_nodes.len() {
        // Get parent context indices for this page
        let parent_indices = get_parent_indices_at(tree_nodes, current_index);
        let parent_count = parent_indices.len();

        // Calculate how many items can fit on this page
        let available_slots = page_height.saturating_sub(parent_count).max(1);
        let remaining_items = tree_nodes.len() - current_index;
        let item_count = available_slots.min(remaining_items);

        page_table.push(PageInfo {
            start_index: current_index,
            item_count,
            parent_indices,
        });

        current_index += item_count;
    }

    page_table
}

/// Get parent context indices for a given start index (top-down order)
fn get_parent_indices_at(tree_nodes: &[TreeNode], start_index: usize) -> Vec<usize> {
    if start_index >= tree_nodes.len() {
        return Vec::new();
    }

    let first_node = &tree_nodes[start_index];
    if let Some(parent_id) = &first_node.pea.parent {
        // Build the parent chain (stores indices)
        let mut parent_indices = Vec::new();
        let mut current_parent_id = Some(parent_id.clone());

        while let Some(pid) = current_parent_id {
            if let Some(parent_index) = tree_nodes.iter().position(|n| n.pea.id == pid) {
                // Check if this parent would be visible in items starting from start_index
                // A parent is visible if it appears at or after start_index
                if parent_index >= start_index {
                    // Parent is on or after this page, no need for context
                    break;
                }
                parent_indices.push(parent_index);
                current_parent_id = tree_nodes[parent_index].pea.parent.clone();
            } else {
                break;
            }
        }

        // Reverse to get top-down order (root ancestor first)
        parent_indices.reverse();
        return parent_indices;
    }
    Vec::new()
}
