// src/interface/handlers/chat.rs

use axum::{Json, extract::State};
use std::sync::Arc;
use rig::{
    completion::Prompt, 
    providers::openai::{self, OpenAIResponsesExt}, // Importamos Ext
    client::CompletionClient // ¡IMPORTANTE! Trait para .agent()
};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE}; // Necesario para Headers
use secrecy::ExposeSecret; 
use crate::domain::{models::{ChatRequest, ChatResponse}, errors::AppError};
use super::admin::AppState;

#[utoipa::path(
    post,
    path = "/api/chat",
    request_body = ChatRequest,
    responses(
        (status = 200, description = "Respuesta RAG Semántico", body = ChatResponse),
        (status = 500, description = "Error interno")
    ),
    tag = "chat"
)]
pub async fn chat_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ChatRequest>,
) -> Result<Json<ChatResponse>, AppError> {
    
    // 1. Obtener lock de lectura del servicio IA
    let ai_guard = state.ai_service.read().await;

    // 2. Generar Embedding
    let embedding = ai_guard.generate_embedding(&payload.message).await?;
    
    // 3. Recuperación Híbrida en Neo4j
    let hybrid_contexts = state.repo.find_hybrid_context(embedding, 3).await?;
    
    // 4. Construir Contexto
    let mut context_text = String::new();
    let mut references_meta = Vec::new();

    for ctx in &hybrid_contexts {
        let entity_list = ctx.connected_entities.join(", ");
        context_text.push_str(&format!(
            "FRAGMENTO [ID: {}]\nCONTENIDO: {}\nCONCEPTOS: [{}]\n---\n", 
            ctx.chunk_id, ctx.content, entity_list
        ));
        let short_id = if ctx.chunk_id.len() > 8 { &ctx.chunk_id[..8] } else { &ctx.chunk_id };
        references_meta.push(format!("Fragmento {} (Conceptos: {})", short_id, entity_list));
    }

    let system_prompt = format!(r#"Eres 'La Muralla'. Responde a la pregunta basándote SOLO en el contexto.
Reglas:
1. Usa [[Concepto]] para entidades del grafo.
2. Cita fuentes como (Ref: ID_FRAGMENTO).
3. Sé conciso.

CONTEXTO:
{}"#, context_text);

    // 5. Configurar el cliente LLM dinámicamente
    let config = ai_guard.get_config(); 
    let base_url = config.base_url.as_deref().unwrap_or("https://api.openai.com/v1");
    let api_key = config.api_key.expose_secret();

    // Construcción manual del cliente (Reemplazo de from_url)
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    
    if !api_key.is_empty() {
        if let Ok(mut val) = HeaderValue::from_str(&format!("Bearer {}", api_key)) {
            val.set_sensitive(true);
            headers.insert(AUTHORIZATION, val);
        }
    }

    let client = openai::Client::from_parts(
        base_url.to_string(),
        headers,
        reqwest::Client::new(),
        OpenAIResponsesExt,
    );

    let agent = client.agent(&config.model_name)
        .preamble(&system_prompt)
        .build();

    let answer = agent.prompt(&payload.message).await
        .map_err(|e| AppError::AIError(format!("Error generando respuesta: {}", e)))?;

    Ok(Json(ChatResponse {
        response: answer,
        context_used: references_meta,
    }))
}