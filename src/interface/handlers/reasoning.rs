use axum::{Json, extract::State};
use std::sync::Arc;
use crate::application::reasoning::ReasoningService;
use crate::domain::models::InferredRelation;
use crate::domain::errors::AppError;
use super::admin::AppState;

#[utoipa::path(
    post,
    path = "/api/reasoning/run",
    responses(
        (status = 200, description = "Knowledge consolidated", body = Vec<InferredRelation>)
    )
)]
pub async fn run_reasoning(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<InferredRelation>>, AppError> {
    
    let service = ReasoningService::new(state.repo.clone(), state.ai_service.clone());
    let new_relations = service.infer_new_knowledge().await?;
    
    Ok(Json(new_relations))
}