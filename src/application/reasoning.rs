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
        // 1. Obtener contexto más amplio
        let graph_context = self.repo.get_graph_context_for_reasoning(500).await?;

        // 2. Prompt Avanzado de Ontología
        let prompt = format!(
            r#"Actúa como un Ingeniero de Ontologías Senior y experto en Lógica Difusa.
            Analiza las siguientes triplas (Entidad -> Relación -> Entidad) extraídas de un grafo:
            
            {}
            
            TU OBJETIVO: Descubrir conocimiento implícito ("Eslabones Perdidos").
            
            REGLAS DE INFERENCIA:
            1. Transitividad: Si A -> B y B -> C, evalúa si lógicamente A -> C.
            2. Resolución de Entidades: Si "Dr. Juan" y "Juan Perez" parecen ser la misma persona por contexto, sugiere relación "SAME_AS".
            3. Causalidad: Si A "CAUSA" B, y B "IMPLICA" C, entonces A "LLEVA_A" C.
            
            FORMATO DE RESPUESTA (JSON estricto):
            {{
                "new_relations": [
                    {{ 
                        "source": "NombreExactoOrigen", 
                        "target": "NombreExactoDestino", 
                        "relation": "TIPO_RELACION_INFERIDA", 
                        "reasoning": "(Confianza: Alta/Media) Explicación breve de por qué dedujiste esto." 
                    }}
                ]
            }}
            
            IMPORTANTE:
            - Solo genera relaciones con una confianza alta.
            - No inventes entidades que no estén en la lista.
            - Si no encuentras nada seguro, devuelve un array vacío.
            "#, 
            graph_context
        );

        // 3. Consultar IA
        let ai_guard = self.ai.read().await;
        
        // Usamos generate_inference que ya maneja la limpieza de JSON
        let response_json = ai_guard.generate_inference(&prompt).await?;
        
        // 4. Guardar en Base de Datos
        if !response_json.new_relations.is_empty() {
            self.repo.save_inferred_relations(response_json.new_relations.clone()).await?;
        }

        Ok(response_json.new_relations)
    }
}