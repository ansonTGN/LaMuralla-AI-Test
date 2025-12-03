use axum::{Json, extract::{State, Path}};
use std::sync::Arc;
use crate::domain::{models::GraphDataResponse, errors::AppError};
use super::admin::AppState;

#[utoipa::path(
    get,
    path = "/api/graph",
    responses(
        (status = 200, description = "Retrieve full graph for visualization", body = GraphDataResponse),
        (status = 500, description = "Database error")
    ),
    tag = "visualization"
)]
pub async fn get_graph(
    State(state): State<Arc<AppState>>,
) -> Result<Json<GraphDataResponse>, AppError> {
    
    // Llamada al repositorio para el grafo completo
    let graph_data = state.repo.get_full_graph().await?;
    
    Ok(Json(graph_data))
}

#[utoipa::path(
    get,
    path = "/api/graph/concept/{name}",
    params(
        ("name" = String, Path, description = "Concept Entity Name to explore")
    ),
    responses(
        (status = 200, description = "Sub-graph neighborhood for specific concept", body = GraphDataResponse),
        (status = 500, description = "Database error")
    ),
    tag = "visualization"
)]
pub async fn get_concept_neighborhood(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> Result<Json<GraphDataResponse>, AppError> {
    
    // Llamada al repositorio para obtener el nodo y sus vecinos (Requiere implementación en Repo)
    let graph_data = state.repo.get_concept_neighborhood(&name).await?;
    
    Ok(Json(graph_data))
}