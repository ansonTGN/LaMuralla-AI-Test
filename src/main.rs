mod domain;
mod application;
mod infrastructure;
mod interface;

use axum::{routing::{post, get}, Router};
use std::sync::Arc;
use tokio::sync::RwLock;
use neo4rs::Graph;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use tower_http::trace::TraceLayer;
use tower_http::cors::CorsLayer;
use secrecy::SecretString;
use tera::Tera;

// --- IMPORTACIONES DEL DOMINIO ---
use crate::domain::models::*;
// ¡IMPORTANTE! Importamos el Trait para poder usar 'create_indexes'
use crate::domain::ports::KGRepository; 

use crate::infrastructure::ai::rig_client::RigAIService;
use crate::infrastructure::persistence::neo4j_repo::Neo4jRepo;
use crate::interface::handlers::{admin::{self, AppState}, ingest, graph, ui, chat};
use crate::application::dtos::*;

// Documentación OpenAPI (Swagger)
#[derive(OpenApi)]
#[openapi(
    paths(
        interface::handlers::admin::update_config,
        interface::handlers::ingest::ingest_document,
        interface::handlers::graph::get_graph,
        interface::handlers::chat::chat_handler
    ),
    components(
        schemas(
            AIConfig, AIProvider, 
            IngestionRequest, IngestionResponse, 
            AdminConfigPayload,
            VisNode, VisEdge, GraphDataResponse,
            ChatRequest, ChatResponse
        )
    ),
    tags(
        (name = "admin", description = "Administration endpoints"),
        (name = "ingestion", description = "Data ingestion endpoints"),
        (name = "visualization", description = "Graph visual exploration"),
        (name = "chat", description = "Semantic GraphRAG Chat")
    )
)]
struct ApiDoc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Cargar variables de entorno y Logs
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    tracing::info!("🚀 Starting La Muralla Backend...");

    // 2. Configuración de IA Agnóstica (OpenAI / Groq / Ollama)
    let provider_str = std::env::var("AI_PROVIDER").unwrap_or_else(|_| "openai".to_string());
    
    // Fallback: Si no hay AI_API_KEY, busca OPENAI_API_KEY, si no, cadena vacía (para Ollama local)
    let api_key_str = std::env::var("AI_API_KEY")
        .or_else(|_| std::env::var("OPENAI_API_KEY"))
        .unwrap_or_else(|_| "".to_string());

    let model_name = std::env::var("AI_MODEL").unwrap_or_else(|_| "gpt-4o".to_string());
    let embedding_model = std::env::var("AI_EMBEDDING_MODEL").unwrap_or_else(|_| "text-embedding-3-small".to_string());
    
    // Importante: Ollama/Nomic suelen usar 768 dimensiones, OpenAI usa 1536.
    let embedding_dim = std::env::var("AI_EMBEDDING_DIM")
        .unwrap_or_else(|_| "1536".to_string())
        .parse::<usize>()
        .expect("AI_EMBEDDING_DIM must be a number");

    let base_url = std::env::var("AI_BASE_URL").ok(); // Option<String>

    // Mapeamos el string del provider al Enum
    let provider = match provider_str.to_lowercase().as_str() {
        "ollama" => AIProvider::Ollama,
        "groq" => AIProvider::Groq,
        _ => AIProvider::OpenAI,
    };

    tracing::info!("🤖 AI Configuration Loaded:");
    tracing::info!("   - Provider: {:?}", provider);
    tracing::info!("   - Model: {}", model_name);
    tracing::info!("   - Embeddings: {} (Dim: {})", embedding_model, embedding_dim);
    if let Some(url) = &base_url {
        tracing::info!("   - Custom Base URL: {}", url);
    }

    let initial_config = AIConfig {
        provider,
        model_name,
        embedding_model,
        api_key: SecretString::new(api_key_str),
        embedding_dim,
        base_url,
    };

    // 3. Conexión a Base de Datos Neo4j
    let uri = std::env::var("NEO4J_URI").expect("NEO4J_URI required in .env");
    let user = std::env::var("NEO4J_USER").expect("NEO4J_USER required in .env");
    let pass = std::env::var("NEO4J_PASS").expect("NEO4J_PASS required in .env");
    
    tracing::info!("🔌 Connecting to Neo4j at {}", uri);
    let graph = Arc::new(Graph::new(&uri, &user, &pass).await?);
    
    // 4. Inicialización de Servicios (Capas de Arquitectura)
    let repo = Arc::new(Neo4jRepo::new(graph.clone()));
    
    // Aseguramos que los índices existan para la dimensión configurada al inicio
    tracing::info!("🛠️ Ensuring Vector Indexes exist for {} dimensions...", embedding_dim);
    
    // AHORA SÍ FUNCIONARÁ PORQUE 'KGRepository' ESTÁ IMPORTADO
    if let Err(e) = repo.create_indexes(embedding_dim).await {
        tracing::warn!("⚠️ Could not ensure indexes (might already exist or DB isn't ready): {}", e);
    }

    let ai_service = Arc::new(RwLock::new(RigAIService::new(initial_config)));

    // 5. Carga de Plantillas HTML (Frontend)
    tracing::info!("🎨 Loading HTML templates...");
    let tera = match Tera::new("templates/**/*.html") {
        Ok(t) => t,
        Err(e) => {
            tracing::error!("❌ Error parsing templates: {}", e);
            ::std::process::exit(1);
        }
    };

    // 6. Estado Compartido de la App
    let app_state = Arc::new(AppState {
        repo,
        ai_service,
        tera, 
    });

    // 7. Definición de Rutas
    let app = Router::new()
        // Swagger UI
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        
        // API Endpoints
        .route("/api/admin/config", post(admin::update_config))
        .route("/api/ingest", post(ingest::ingest_document))
        .route("/api/graph", get(graph::get_graph))
        .route("/api/chat", post(chat::chat_handler))
        
        // Frontend UI
        .route("/", get(ui::render_dashboard))

        // Middleware
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .with_state(app_state);

    // 8. Arrancar Servidor
    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("0.0.0.0:{}", port);
    
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("✅ Server running on http://{}", addr);
    
    axum::serve(listener, app).await?;

    Ok(())
}