use std::sync::Arc;
use tokio::sync::RwLock;
use crate::domain::{
    ports::{KGRepository, AIService},
    models::InferredRelation,
    errors::AppError
};

pub struct ReasoningService {
    repo: Arc<dyn KGRepository>,
    ai: Arc<RwLock<dyn AIService>>,
}

impl ReasoningService {
    pub fn new(repo: Arc<dyn KGRepository>, ai: Arc<RwLock<dyn AIService>>) -> Self {
        Self { repo, ai }
    }

    pub async fn infer_new_knowledge(&self) -> Result<Vec<InferredRelation>, AppError> {
        // 1. Obtener contexto (limitado para no desbordar tokens)
        let graph_context = self.repo.get_graph_context_for_reasoning(300).await?;

        // 2. Prompt
        let prompt = format!(
            r#"Analiza las siguientes relaciones existentes en un Grafo de Conocimiento:
            
            {}
            
            TU TAREA:
            1. Identifica relaciones IMPLÍCITAS lógicas que falten. (Ej: Si A->B y B->C, ¿A->C?)
            2. Identifica entidades que sean SINÓNIMAS y deban unirse (Relación: SAME_AS).
            3. Genera nuevas conexiones conceptuales basadas en tu conocimiento del mundo real.
            
            Responde ESTRICTAMENTE en JSON con este formato:
            {{
                "new_relations": [
                    {{ "source": "NombreExactoOrigen", "target": "NombreExactoDestino", "relation": "TIPO_RELACION", "reasoning": "Breve explicacion" }}
                ]
            }}
            No inventes entidades nuevas, usa solo las existentes en el texto provisto. Si no hay nada obvio, devuelve array vacío.
            "#, 
            graph_context
        );

        // 3. Consultar IA
        let ai_guard = self.ai.read().await;
        let response_json = ai_guard.generate_inference(&prompt).await?;
        
        // 4. Guardar
        if !response_json.new_relations.is_empty() {
            self.repo.save_inferred_relations(response_json.new_relations.clone()).await?;
        }

        Ok(response_json.new_relations)
    }
}