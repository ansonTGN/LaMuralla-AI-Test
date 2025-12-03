use serde::{Deserialize, Serialize};
use secrecy::SecretString;
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub enum AIProvider {
    OpenAI,
    Ollama,
    Groq,
}

fn default_api_key() -> SecretString {
    SecretString::new("".to_string())
}

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema, Clone)]
pub struct AIConfig {
    pub provider: AIProvider,
    #[validate(length(min = 1))]
    pub model_name: String,
    #[validate(length(min = 1))]
    pub embedding_model: String,
    
    #[serde(skip_serializing, default = "default_api_key")]
    #[schema(value_type = String)] 
    pub api_key: SecretString,
    
    pub embedding_dim: usize,
    #[validate(url)]
    pub base_url: Option<String>, 
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct GraphEntity {
    pub name: String,
    pub category: String, 
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct GraphRelation {
    pub source: String,
    pub target: String,
    pub relation_type: String, 
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KnowledgeExtraction {
    pub entities: Vec<GraphEntity>,
    pub relations: Vec<GraphRelation>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Validate)]
pub struct IngestionRequest {
    #[validate(length(min = 10))]
    pub content: String,
    pub metadata: serde_json::Value,
}

// --- MODELOS DE VISUALIZACIÓN ---

#[derive(Debug, Serialize, ToSchema)]
pub struct VisNode {
    pub id: String,
    pub label: String,
    pub group: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct VisEdge {
    pub from: String,
    pub to: String,
    pub label: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct GraphDataResponse {
    pub nodes: Vec<VisNode>,
    pub edges: Vec<VisEdge>,
}

// --- CHAT RAG ---

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ChatRequest {
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ChatResponse {
    pub response: String,
    pub context_used: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct HybridContext {
    pub chunk_id: String,
    pub content: String,
    pub connected_entities: Vec<String>, 
}

// --- NUEVO: RAZONAMIENTO E INFERENCIA ---

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct InferredRelation {
    pub source: String,
    pub target: String,
    pub relation: String,
    pub reasoning: String, // Explicación de por qué la IA creó esto
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InferenceResult {
    pub new_relations: Vec<InferredRelation>,
}