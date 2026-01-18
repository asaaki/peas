use std::path::PathBuf;
use std::sync::Arc;

use async_graphql::{Context, EmptySubscription, Object, Schema};

use crate::config::PeasConfig;
use crate::model::Pea as ModelPea;
use crate::storage::PeaRepository;

use super::types::*;

pub type PeasSchema = Schema<QueryRoot, MutationRoot, EmptySubscription>;

pub struct AppState {
    pub config: PeasConfig,
    pub project_root: PathBuf,
}

pub fn build_schema(config: PeasConfig, project_root: PathBuf) -> PeasSchema {
    let state = Arc::new(AppState {
        config,
        project_root,
    });

    Schema::build(QueryRoot, MutationRoot, EmptySubscription)
        .data(state)
        .finish()
}

fn get_repo(ctx: &Context<'_>) -> PeaRepository {
    let state = ctx.data::<Arc<AppState>>().unwrap();
    PeaRepository::new(&state.config, &state.project_root)
}

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    /// Get a single pea by ID
    async fn pea(&self, ctx: &Context<'_>, id: String) -> async_graphql::Result<Option<Pea>> {
        let repo = get_repo(ctx);
        match repo.get(&id) {
            Ok(pea) => Ok(Some(pea.into())),
            Err(crate::error::PeasError::NotFound(_)) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// List peas with optional filtering
    async fn peas(
        &self,
        ctx: &Context<'_>,
        filter: Option<PeaFilter>,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> async_graphql::Result<PeaConnection> {
        let repo = get_repo(ctx);
        let mut peas = repo.list()?;

        // Apply filters
        if let Some(f) = filter {
            if let Some(t) = f.pea_type {
                let filter_type: crate::model::PeaType = t.into();
                peas.retain(|p| p.pea_type == filter_type);
            }
            if let Some(s) = f.status {
                let filter_status: crate::model::PeaStatus = s.into();
                peas.retain(|p| p.status == filter_status);
            }
            if let Some(p) = f.priority {
                let filter_priority: crate::model::PeaPriority = p.into();
                peas.retain(|pea| pea.priority == filter_priority);
            }
            if let Some(ref parent_id) = f.parent {
                peas.retain(|p| p.parent.as_deref() == Some(parent_id.as_str()));
            }
            if let Some(ref tag) = f.tag {
                peas.retain(|p| p.tags.contains(tag));
            }
            if let Some(is_open) = f.is_open {
                peas.retain(|p| p.is_open() == is_open);
            }
        }

        let total_count = peas.len();

        // Apply pagination
        let offset = offset.unwrap_or(0);
        let limit = limit.unwrap_or(100);
        let peas: Vec<Pea> = peas
            .into_iter()
            .skip(offset)
            .take(limit)
            .map(|p| p.into())
            .collect();

        Ok(PeaConnection {
            nodes: peas,
            total_count,
        })
    }

    /// Search peas by text in title and body
    async fn search(
        &self,
        ctx: &Context<'_>,
        query: String,
        limit: Option<usize>,
    ) -> async_graphql::Result<Vec<Pea>> {
        let repo = get_repo(ctx);
        let peas = repo.list()?;
        let query_lower = query.to_lowercase();

        let results: Vec<Pea> = peas
            .into_iter()
            .filter(|p| {
                p.title.to_lowercase().contains(&query_lower)
                    || p.body.to_lowercase().contains(&query_lower)
                    || p.id.to_lowercase().contains(&query_lower)
            })
            .take(limit.unwrap_or(50))
            .map(|p| p.into())
            .collect();

        Ok(results)
    }

    /// Get children of a pea
    async fn children(
        &self,
        ctx: &Context<'_>,
        parent_id: String,
    ) -> async_graphql::Result<Vec<Pea>> {
        let repo = get_repo(ctx);
        let children = repo.find_children(&parent_id)?;
        Ok(children.into_iter().map(|p| p.into()).collect())
    }

    /// Get project statistics
    async fn stats(&self, ctx: &Context<'_>) -> async_graphql::Result<ProjectStats> {
        let repo = get_repo(ctx);
        let peas = repo.list()?;

        use crate::model::{PeaStatus as MS, PeaType as MT};

        Ok(ProjectStats {
            total: peas.len(),
            by_status: StatusCounts {
                draft: peas.iter().filter(|p| p.status == MS::Draft).count(),
                todo: peas.iter().filter(|p| p.status == MS::Todo).count(),
                in_progress: peas.iter().filter(|p| p.status == MS::InProgress).count(),
                completed: peas.iter().filter(|p| p.status == MS::Completed).count(),
                scrapped: peas.iter().filter(|p| p.status == MS::Scrapped).count(),
            },
            by_type: TypeCounts {
                milestone: peas.iter().filter(|p| p.pea_type == MT::Milestone).count(),
                epic: peas.iter().filter(|p| p.pea_type == MT::Epic).count(),
                feature: peas.iter().filter(|p| p.pea_type == MT::Feature).count(),
                bug: peas.iter().filter(|p| p.pea_type == MT::Bug).count(),
                task: peas.iter().filter(|p| p.pea_type == MT::Task).count(),
            },
        })
    }
}

pub struct MutationRoot;

#[Object]
impl MutationRoot {
    /// Create a new pea
    async fn create_pea(
        &self,
        ctx: &Context<'_>,
        input: CreatePeaInput,
    ) -> async_graphql::Result<Pea> {
        let repo = get_repo(ctx);
        let id = repo.generate_id();

        let pea_type = input.pea_type.map(|t| t.into()).unwrap_or_default();
        let mut pea = ModelPea::new(id, input.title, pea_type);

        if let Some(s) = input.status {
            pea = pea.with_status(s.into());
        }
        if let Some(p) = input.priority {
            pea = pea.with_priority(p.into());
        }
        if let Some(b) = input.body {
            pea = pea.with_body(b);
        }
        if input.parent.is_some() {
            pea = pea.with_parent(input.parent);
        }
        if let Some(blocking) = input.blocking {
            pea = pea.with_blocking(blocking);
        }
        if let Some(tags) = input.tags {
            pea = pea.with_tags(tags);
        }

        repo.create(&pea)?;
        Ok(pea.into())
    }

    /// Update an existing pea
    async fn update_pea(
        &self,
        ctx: &Context<'_>,
        input: UpdatePeaInput,
    ) -> async_graphql::Result<Pea> {
        let repo = get_repo(ctx);
        let mut pea = repo.get(&input.id)?;

        if let Some(title) = input.title {
            pea.title = title;
        }
        if let Some(t) = input.pea_type {
            pea.pea_type = t.into();
        }
        if let Some(s) = input.status {
            pea.status = s.into();
        }
        if let Some(p) = input.priority {
            pea.priority = p.into();
        }
        if let Some(body) = input.body {
            pea.body = body;
        }
        if let Some(parent) = input.parent {
            pea.parent = if parent.is_empty() {
                None
            } else {
                Some(parent)
            };
        }
        if let Some(blocking) = input.blocking {
            pea.blocking = blocking;
        }
        if let Some(tags) = input.add_tags {
            for tag in tags {
                if !pea.tags.contains(&tag) {
                    pea.tags.push(tag);
                }
            }
        }
        if let Some(tags) = input.remove_tags {
            for tag in tags {
                pea.tags.retain(|t| t != &tag);
            }
        }

        pea.touch();
        repo.update(&pea)?;
        Ok(pea.into())
    }

    /// Set the status of a pea
    async fn set_status(
        &self,
        ctx: &Context<'_>,
        id: String,
        status: PeaStatus,
    ) -> async_graphql::Result<Pea> {
        let repo = get_repo(ctx);
        let mut pea = repo.get(&id)?;
        pea.status = status.into();
        pea.touch();
        repo.update(&pea)?;
        Ok(pea.into())
    }

    /// Archive a pea
    async fn archive_pea(&self, ctx: &Context<'_>, id: String) -> async_graphql::Result<bool> {
        let repo = get_repo(ctx);
        repo.archive(&id)?;
        Ok(true)
    }

    /// Delete a pea permanently
    async fn delete_pea(&self, ctx: &Context<'_>, id: String) -> async_graphql::Result<bool> {
        let repo = get_repo(ctx);
        repo.delete(&id)?;
        Ok(true)
    }

    /// Add a tag to a pea
    async fn add_tag(
        &self,
        ctx: &Context<'_>,
        id: String,
        tag: String,
    ) -> async_graphql::Result<Pea> {
        let repo = get_repo(ctx);
        let mut pea = repo.get(&id)?;
        if !pea.tags.contains(&tag) {
            pea.tags.push(tag);
            pea.touch();
            repo.update(&pea)?;
        }
        Ok(pea.into())
    }

    /// Remove a tag from a pea
    async fn remove_tag(
        &self,
        ctx: &Context<'_>,
        id: String,
        tag: String,
    ) -> async_graphql::Result<Pea> {
        let repo = get_repo(ctx);
        let mut pea = repo.get(&id)?;
        pea.tags.retain(|t| t != &tag);
        pea.touch();
        repo.update(&pea)?;
        Ok(pea.into())
    }
}
