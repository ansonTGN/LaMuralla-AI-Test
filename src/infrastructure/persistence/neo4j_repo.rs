use async_trait::async_trait;
use neo4rs::{Graph, query};
use uuid::Uuid;
use std::sync::Arc;
use std::collections::HashSet;
use crate::domain::{
    ports::KGRepository, 
    models::{KnowledgeExtraction, GraphDataResponse, VisNode, VisEdge, HybridContext, InferredRelation}, 
    errors::AppError
};

pub struct Neo4jRepo {
    graph: Arc<Graph>,
}

impl Neo4jRepo {
    pub fn new(graph: Arc<Graph>) -> Self {
        Self { graph }
    }
}

#[async_trait]
impl KGRepository for Neo4jRepo {
    async fn create_indexes(&self, dim: usize) -> Result<(), AppError> {
        let q = format!(
            "CREATE VECTOR INDEX chunk_embeddings IF NOT EXISTS FOR (c:DocumentChunk) ON (c.embedding) \
             OPTIONS {{indexConfig: {{ `vector.dimensions`: {}, `vector.similarity_function`: 'cosine' }} }}", 
            dim
        );
        self.graph.run(query(&q)).await.map_err(|e| AppError::DatabaseError(e.to_string()))?;
        
        self.graph.run(query("CREATE CONSTRAINT entity_name IF NOT EXISTS FOR (e:Entity) REQUIRE e.name IS UNIQUE")).await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
            
        Ok(())
    }

    async fn reset_database(&self) -> Result<(), AppError> {
        self.graph.run(query("MATCH (n) DETACH DELETE n")).await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn save_chunk(&self, id: Uuid, content: &str, embedding: Vec<f32>) -> Result<(), AppError> {
        let q = query("CREATE (c:DocumentChunk {id: $id, content: $content, embedding: $embedding})")
            .param("id", id.to_string())
            .param("content", content)
            .param("embedding", embedding);
        
        self.graph.run(q).await.map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn save_graph(&self, chunk_id: Uuid, data: KnowledgeExtraction) -> Result<(), AppError> {
        let mut txn = self.graph.start_txn().await.map_err(|e| AppError::DatabaseError(e.to_string()))?;

        for entity in &data.entities {
            let q = query("MERGE (e:Entity {name: $name}) ON CREATE SET e.category = $category")
                .param("name", entity.name.as_str())
                .param("category", entity.category.as_str());
            txn.run(q).await.map_err(|e| AppError::DatabaseError(e.to_string()))?;
        }

        for rel in data.relations {
            let cypher = format!(
                "MATCH (a:Entity {{name: $source}}), (b:Entity {{name: $target}}) \
                 MERGE (a)-[:{}]->(b)", 
                rel.relation_type.replace(" ", "_").to_uppercase() 
            );
            let q = query(&cypher)
                .param("source", rel.source.as_str())
                .param("target", rel.target.as_str());
            txn.run(q).await.map_err(|e| AppError::DatabaseError(e.to_string()))?;
        }

        let q_link = query("MATCH (c:DocumentChunk {id: $cid}), (e:Entity) \
                            WHERE e.name IN $names \
                            MERGE (c)-[:MENTIONS]->(e)");
        
        let names: Vec<String> = data.entities.into_iter().map(|e| e.name).collect();
        txn.run(q_link.param("cid", chunk_id.to_string()).param("names", names)).await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        txn.commit().await.map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn get_full_graph(&self) -> Result<GraphDataResponse, AppError> {
        let q = query(
            "MATCH (n:Entity)-[r]->(m:Entity) \
             RETURN n.name, n.category, type(r), m.name, m.category \
             LIMIT 1000"
        );
        
        let mut stream = self.graph.execute(q).await.map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let mut nodes_vec = Vec::new();
        let mut edges_vec = Vec::new();
        let mut unique_nodes = HashSet::new(); 

        while let Ok(Some(row)) = stream.next().await {
            let n_name: String = row.get("n.name").unwrap_or_else(|_| "Unknown".to_string());
            let n_cat: String = row.get("n.category").unwrap_or_else(|_| "Concept".to_string());
            let r_type: String = row.get("type(r)").unwrap_or_else(|_| "RELATED".to_string());
            let m_name: String = row.get("m.name").unwrap_or_else(|_| "Unknown".to_string());
            let m_cat: String = row.get("m.category").unwrap_or_else(|_| "Concept".to_string());

            if unique_nodes.insert(n_name.clone()) {
                nodes_vec.push(VisNode { id: n_name.clone(), label: n_name.clone(), group: n_cat });
            }
            if unique_nodes.insert(m_name.clone()) {
                nodes_vec.push(VisNode { id: m_name.clone(), label: m_name.clone(), group: m_cat });
            }

            edges_vec.push(VisEdge { from: n_name, to: m_name, label: r_type });
        }

        Ok(GraphDataResponse { nodes: nodes_vec, edges: edges_vec })
    }

    async fn find_hybrid_context(&self, embedding: Vec<f32>, limit: usize) -> Result<Vec<HybridContext>, AppError> {
        let q_str = format!(
            "CALL db.index.vector.queryNodes('chunk_embeddings', {}, $embedding) \
             YIELD node as chunk, score \
             MATCH (chunk)-[:MENTIONS]->(e:Entity) \
             RETURN chunk.id as id, chunk.content as content, collect(DISTINCT e.name) as entities", 
            limit
        );

        let q = query(&q_str).param("embedding", embedding);
        let mut stream = self.graph.execute(q).await.map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let mut results = Vec::new();
        while let Ok(Some(row)) = stream.next().await {
            let id: String = row.get("id").unwrap_or_else(|_| "unk".to_string());
            let content: String = row.get("content").unwrap_or_default();
            let entities: Vec<String> = row.get("entities").unwrap_or_default();

            results.push(HybridContext {
                chunk_id: id,
                content,
                connected_entities: entities,
            });
        }
        
        Ok(results)
    }
    
    // --- IMPLEMENTACIÓN: VECINDARIO DE CONCEPTO (Deep Dive) ---

    async fn get_concept_neighborhood(&self, concept_name: &str) -> Result<GraphDataResponse, AppError> {
        // Busca el nodo central y todas las relaciones (entrantes o salientes) directas
        let q = query(
            "MATCH (center:Entity {name: $name})-[r]-(neighbor:Entity)
             RETURN center.name, center.category, type(r) as rel, startNode(r) = center as is_source, neighbor.name, neighbor.category
             LIMIT 100"
        ).param("name", concept_name);

        let mut stream = self.graph.execute(q).await.map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let mut nodes_vec = Vec::new();
        let mut edges_vec = Vec::new();
        let mut unique_nodes = HashSet::new();

        let mut relations_found = false;

        while let Ok(Some(row)) = stream.next().await {
            relations_found = true;
            
            let c_name: String = row.get("center.name").unwrap_or_default();
            let c_cat: String = row.get("center.category").unwrap_or_else(|_| "Concept".to_string());
            let rel_type: String = row.get("rel").unwrap_or_default();
            let is_source: bool = row.get("is_source").unwrap_or(true);
            let n_name: String = row.get("neighbor.name").unwrap_or_default();
            let n_cat: String = row.get("neighbor.category").unwrap_or_else(|_| "Concept".to_string());

            // Añadir/Actualizar nodo central
            if unique_nodes.insert(c_name.clone()) {
                 nodes_vec.push(VisNode { id: c_name.clone(), label: c_name.clone(), group: c_cat });
            }

            // Añadir nodo vecino
            if unique_nodes.insert(n_name.clone()) {
                nodes_vec.push(VisNode { id: n_name.clone(), label: n_name.clone(), group: n_cat });
            }

            // Definir dirección
            let (from, to) = if is_source {
                (c_name.clone(), n_name.clone())
            } else {
                (n_name.clone(), c_name.clone())
            };

            edges_vec.push(VisEdge { from, to, label: rel_type });
        }
        
        // Fallback: Si no hay relaciones, al menos devolvemos el nodo central
        if !relations_found {
             let q_fallback = query("MATCH (center:Entity {name: $name}) RETURN center.name, center.category")
                .param("name", concept_name);
             let mut stream_fallback = self.graph.execute(q_fallback).await.map_err(|e| AppError::DatabaseError(e.to_string()))?;
             if let Ok(Some(row)) = stream_fallback.next().await {
                let name: String = row.get("center.name").unwrap_or_default();
                let cat: String = row.get("center.category").unwrap_or_else(|_| "Concept".to_string());
                nodes_vec.push(VisNode { id: name.clone(), label: name, group: cat });
             }
        }

        // Limpiar duplicados de nodos (si se insertó dos veces en el loop principal o fallback)
        nodes_vec.sort_by(|a, b| a.id.cmp(&b.id));
        nodes_vec.dedup_by(|a, b| a.id == b.id);

        Ok(GraphDataResponse { nodes: nodes_vec, edges: edges_vec })
    }
    
    // --- MÉTODOS DE RAZONAMIENTO (EXISTENTES) ---

    async fn get_graph_context_for_reasoning(&self, limit: usize) -> Result<String, AppError> {
        // Obtenemos las relaciones más "densas" para dar contexto
        let q = query(
            "MATCH (n:Entity)-[r]->(m:Entity) 
             WITH n, r, m, count(n) as degree 
             ORDER BY degree DESC 
             LIMIT $limit 
             RETURN n.name, type(r), m.name"
        ).param("limit", limit as i64);

        let mut stream = self.graph.execute(q).await.map_err(|e| AppError::DatabaseError(e.to_string()))?;
        let mut context = String::new();

        while let Ok(Some(row)) = stream.next().await {
            let n: String = row.get("n.name").unwrap_or_default();
            let r: String = row.get("type(r)").unwrap_or_default();
            let m: String = row.get("m.name").unwrap_or_default();
            context.push_str(&format!("({}) -[{}]-> ({})\n", n, r, m));
        }

        if context.is_empty() {
            return Ok("El grafo está vacío.".to_string());
        }
        Ok(context)
    }

    async fn save_inferred_relations(&self, relations: Vec<InferredRelation>) -> Result<(), AppError> {
        let mut txn = self.graph.start_txn().await.map_err(|e| AppError::DatabaseError(e.to_string()))?;

        for rel in relations {
            let cypher = format!(
                "MATCH (a:Entity {{name: $source}}), (b:Entity {{name: $target}}) \
                 MERGE (a)-[r:INFERRED_{}]->(b) \
                 ON CREATE SET r.reasoning = $reasoning, r.is_ai_generated = true",
                rel.relation.replace(" ", "_").to_uppercase()
            );
            
            let q = query(&cypher)
                .param("source", rel.source)
                .param("target", rel.target)
                .param("reasoning", rel.reasoning);
                
            txn.run(q).await.map_err(|e| AppError::DatabaseError(e.to_string()))?;
        }

        txn.commit().await.map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(())
    }
}