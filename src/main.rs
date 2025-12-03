mod domain;
mod application;
mod infrastructure;
mod interface;

use axum::{
    routing::{post, get}, 
    Router, 
    response::{Redirect, IntoResponse}, // <-- Redirect e IntoResponse añadidos aquí
}; 
use std::sync::Arc;
use tokio::sync::RwLock;
use neo4rs::Graph;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use tower_http::trace::TraceLayer;
use tower_http::cors::CorsLayer;
use secrecy::SecretString;
use tera::Tera;

use crate::domain::models::*;
use crate::domain::ports::KGRepository; 

use crate::infrastructure::ai::rig_client::RigAIService;
use crate::infrastructure::persistence::neo4j_repo::Neo4jRepo;
use crate::interface::handlers::{admin::{self, AppState}, ingest, graph, ui, chat, reasoning}; 
use crate::application::dtos::*;

// Documentación OpenAPI (Swagger)
#[derive(OpenApi)]
#[openapi(
    paths(
        interface::handlers::admin::update_config,
        interface::handlers::ingest::ingest_document,
        interface::handlers::graph::get_graph,
        interface::handlers::chat::chat_handler,
        interface::handlers::reasoning::run_reasoning
    ),
    components(
        schemas(
            AIConfig, AIProvider, 
            IngestionRequest, IngestionResponse, 
            AdminConfigPayload,
            VisNode, VisEdge, GraphDataResponse,
            ChatRequest, ChatResponse, // ChatResponse modificado en models.rs
            InferredRelation 
        )
    ),
    tags(
        (name = "admin", description = "Administration endpoints"),
        (name = "ingestion", description = "Data ingestion endpoints"),
        (name = "visualization", description = "Graph visual exploration"),
        (name = "chat", description = "Semantic GraphRAG Chat"),
        (name = "reasoning", description = "AI Graph Enrichment")
    )
)]
struct ApiDoc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    tracing::info!("🚀 Starting La Muralla Backend...");

    let provider_str = std::env::var("AI_PROVIDER").unwrap_or_else(|_| "openai".to_string());
    let api_key_str = std::env::var("AI_API_KEY")
        .or_else(|_| std::env::var("OPENAI_API_KEY"))
        .unwrap_or_else(|_| "".to_string());

    let model_name = std::env::var("AI_MODEL").unwrap_or_else(|_| "gpt-4o".to_string());
    let embedding_model = std::env::var("AI_EMBEDDING_MODEL").unwrap_or_else(|_| "text-embedding-3-small".to_string());
    let embedding_dim = std::env::var("AI_EMBEDDING_DIM")
        .unwrap_or_else(|_| "1536".to_string())
        .parse::<usize>()
        .expect("AI_EMBEDDING_DIM must be a number");
    let base_url = std::env::var("AI_BASE_URL").ok();

    let provider = match provider_str.to_lowercase().as_str() {
        "ollama" => AIProvider::Ollama,
        "groq" => AIProvider::Groq,
        _ => AIProvider::OpenAI,
    };

    let initial_config = AIConfig {
        provider,
        model_name,
        embedding_model,
        api_key: SecretString::new(api_key_str),
        embedding_dim,
        base_url,
    };

    let uri = std::env::var("NEO4J_URI").expect("NEO4J_URI required in .env");
    let user = std::env::var("NEO4J_USER").expect("NEO4J_USER required in .env");
    let pass = std::env::var("NEO4J_PASS").expect("NEO4J_PASS required in .env");
    
    tracing::info!("🔌 Connecting to Neo4j at {}", uri);
    let graph = Arc::new(Graph::new(&uri, &user, &pass).await?);
    
    let repo = Arc::new(Neo4jRepo::new(graph.clone()));
    
    if let Err(e) = repo.create_indexes(embedding_dim).await {
        tracing::warn!("⚠️ Could not ensure indexes: {}", e);
    }

    let ai_service = Arc::new(RwLock::new(RigAIService::new(initial_config)));

    let tera = match Tera::new("templates/**/*.html") {
        Ok(t) => t,
        Err(e) => {
            tracing::error!("❌ Error parsing templates: {}", e);
            ::std::process::exit(1);
        }
    };

    let app_state = Arc::new(AppState {
        repo,
        ai_service,
        tera, 
    });

    let app = Router::new()
        // API (sin protección, se asume que las llamadas vienen del frontend ya autenticado)
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .route("/api/admin/config", post(admin::update_config))
        .route("/api/ingest", post(ingest::ingest_document))
        .route("/api/graph", get(graph::get_graph))
        .route("/api/chat", post(chat::chat_handler))
        .route("/api/reasoning/run", post(reasoning::run_reasoning))
        
        // INTERFAZ DE USUARIO (Protegida)
        .route("/", get(ui::render_login).post(ui::authenticate)) // Pantalla de Login y handler POST
        .route("/dashboard", get(ui::render_dashboard_guarded)) // Dashboard, protegido por el guard
        .route("/logout", get(|| async { Redirect::to("/").into_response() })) // Logout simple
        
        // Capas de Axum
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .with_state(app_state);

    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("0.0.0.0:{}", port);
    
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("✅ Server running on http://{}", addr);
    
    axum::serve(listener, app).await?;

    Ok(())
}