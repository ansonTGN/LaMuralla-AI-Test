use async_trait::async_trait;
use crate::domain::models::{AIConfig, KnowledgeExtraction, GraphDataResponse, HybridContext, InferredRelation, InferenceResult};
use crate::domain::errors::AppError;
use uuid::Uuid;

#[async_trait]
pub trait KGRepository: Send + Sync {
    async fn save_chunk(&self, id: Uuid, content: &str, embedding: Vec<f32>) -> Result<(), AppError>;
    async fn save_graph(&self, chunk_id: Uuid, data: KnowledgeExtraction) -> Result<(), AppError>;
    async fn reset_database(&self) -> Result<(), AppError>;
    async fn create_indexes(&self, dim: usize) -> Result<(), AppError>;
    
    async fn get_full_graph(&self) -> Result<GraphDataResponse, AppError>;
    async fn find_hybrid_context(&self, embedding: Vec<f32>, limit: usize) -> Result<Vec<HybridContext>, AppError>;

    // --- NUEVO: Métodos para razonamiento ---
    async fn get_graph_context_for_reasoning(&self, limit: usize) -> Result<String, AppError>;
    async fn save_inferred_relations(&self, relations: Vec<InferredRelation>) -> Result<(), AppError>;
}

#[async_trait]
pub trait AIService: Send + Sync {
    async fn extract_knowledge(&self, text: &str) -> Result<KnowledgeExtraction, AppError>;
    async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>, AppError>;
    fn update_config(&mut self, config: AIConfig) -> Result<(), AppError>;
    fn get_config(&self) -> AIConfig;

    // --- NUEVO: Método para inferencia ---
    async fn generate_inference(&self, prompt: &str) -> Result<InferenceResult, AppError>;
}