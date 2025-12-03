use uuid::Uuid;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::domain::{
    ports::{KGRepository, AIService},
    // models::IngestionRequest, // Comentado para evitar warning
    errors::AppError
};

// Tamaño máximo aproximado por chunk (en caracteres).
// 24.000 caracteres son aprox 6.000 tokens (seguro para el límite de 8.192).
const CHUNK_SIZE: usize = 24_000;

pub struct IngestionService {
    repo: Arc<dyn KGRepository>,
    ai: Arc<RwLock<dyn AIService>>,
}

impl IngestionService {
    pub fn new(repo: Arc<dyn KGRepository>, ai: Arc<RwLock<dyn AIService>>) -> Self {
        Self { repo, ai }
    }

    /// Función auxiliar para dividir texto preservando palabras completas
    fn split_text_into_chunks(&self, text: &str) -> Vec<String> {
        let mut chunks = Vec::new();
        let mut current_chunk = String::new();

        // Dividimos por espacios para no cortar palabras
        for word in text.split_inclusive(char::is_whitespace) {
            if current_chunk.len() + word.len() > CHUNK_SIZE {
                if !current_chunk.is_empty() {
                    chunks.push(current_chunk);
                    current_chunk = String::new();
                }
            }
            current_chunk.push_str(word);
        }
        if !current_chunk.is_empty() {
            chunks.push(current_chunk);
        }
        
        // Si el texto era muy corto y no entró al loop
        if chunks.is_empty() && !text.is_empty() {
            chunks.push(text.to_string());
        }

        chunks
    }

    pub async fn ingest_with_progress(
        &self, 
        content: String,
        progress_tx: tokio::sync::mpsc::Sender<String>
    ) -> Result<Uuid, AppError> {
        
        // 1. Dividir el contenido en trozos (Chunks)
        let chunks = self.split_text_into_chunks(&content);
        let total_chunks = chunks.len();
        let doc_group_id = Uuid::new_v4(); // ID para agrupar (opcional en lógica futura)

        let _ = progress_tx.send(format!("🔪 Documento largo detectado. Dividido en {} fragmentos.", total_chunks)).await;

        // 2. Procesar cada chunk
        for (index, chunk_text) in chunks.iter().enumerate() {
            let current_step = index + 1;
            let chunk_id = Uuid::new_v4();

            // A. Vectorizar
            let _ = progress_tx.send(format!("🧠 [{}/{}] Generando Embeddings...", current_step, total_chunks)).await;
            
            // Obtenemos lock para IA
            let ai_guard = self.ai.read().await;
            
            // Manejo de error específico de Embeddings para no detener todo el proceso si uno falla
            let embedding = match ai_guard.generate_embedding(chunk_text).await {
                Ok(emb) => emb,
                Err(e) => {
                    let _ = progress_tx.send(format!("⚠️ Error embedding chunk {}: {}. Saltando...", current_step, e)).await;
                    continue; 
                }
            };

            // B. Guardar Chunk
            // let _ = progress_tx.send(format!("💾 [{}/{}] Guardando datos...", current_step, total_chunks)).await;
            self.repo.save_chunk(chunk_id, chunk_text, embedding).await?;

            // C. Extracción Simbólica (LLM)
            let _ = progress_tx.send(format!("🕵️ [{}/{}] Extrayendo conocimiento...", current_step, total_chunks)).await;
            
            match ai_guard.extract_knowledge(chunk_text).await {
                Ok(extraction) => {
                    let count = extraction.entities.len();
                    let _ = progress_tx.send(format!("🕸️ [{}/{}] Conectando {} entidades al grafo...", current_step, total_chunks, count)).await;
                    self.repo.save_graph(chunk_id, extraction).await?;
                },
                Err(e) => {
                    let _ = progress_tx.send(format!("⚠️ Error extrayendo entidades en parte {}: {}", current_step, e)).await;
                    // No detenemos el proceso, solo avisamos
                }
            };
        }

        let _ = progress_tx.send("✅ ¡Todo el documento ha sido procesado!".to_string()).await;

        // Retornamos el ID del último chunk procesado (o uno nuevo genérico)
        Ok(doc_group_id)
    }
}