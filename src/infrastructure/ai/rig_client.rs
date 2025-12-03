use async_trait::async_trait;
use rig::{
    providers::openai::{self, OpenAIResponsesExt},
    client::{CompletionClient, EmbeddingsClient},
    completion::Prompt,
    embeddings::EmbeddingsBuilder,
};
use secrecy::ExposeSecret;
use serde_json::from_str;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use crate::domain::{models::{AIConfig, KnowledgeExtraction, InferenceResult}, ports::AIService, errors::AppError};

pub struct RigAIService {
    config: AIConfig,
}

impl RigAIService {
    pub fn new(config: AIConfig) -> Self {
        Self { config }
    }

    fn clean_json_response(&self, raw: &str) -> String {
        raw.trim()
            .trim_start_matches("```json")
            .trim_start_matches("```")
            .trim_end_matches("```")
            .to_string()
    }
    
    fn get_client(&self) -> openai::Client {
        let base_url = self.config.base_url.as_deref().unwrap_or("https://api.openai.com/v1");
        let api_key = self.config.api_key.expose_secret();

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        
        if !api_key.is_empty() {
            if let Ok(mut val) = HeaderValue::from_str(&format!("Bearer {}", api_key)) {
                val.set_sensitive(true);
                headers.insert(AUTHORIZATION, val);
            }
        }

        openai::Client::from_parts(
            base_url.to_string(),
            headers,
            reqwest::Client::new(),
            OpenAIResponsesExt,
        )
    }
}

#[async_trait]
impl AIService for RigAIService {
    fn update_config(&mut self, config: AIConfig) -> Result<(), AppError> {
        self.config = config;
        Ok(())
    }

    fn get_config(&self) -> AIConfig {
        self.config.clone()
    }

    async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>, AppError> {
        let client = self.get_client(); 
        let model = client.embedding_model(&self.config.embedding_model);
        
        let embeddings = EmbeddingsBuilder::new(model)
            .document(text) 
            .map_err(|e| AppError::AIError(format!("Error adding document: {}", e)))? 
            .build()
            .await
            .map_err(|e| AppError::AIError(format!("Embedding failed (Provider: {:?}): {}", self.config.provider, e)))?;

        let (_, embedding_data) = embeddings.first()
            .ok_or_else(|| AppError::AIError("No embedding returned".to_string()))?;
            
        let first_embedding = embedding_data.first();
        let embedding_f32: Vec<f32> = first_embedding.vec.iter().map(|&x| x as f32).collect();
        
        Ok(embedding_f32)
    }

    async fn extract_knowledge(&self, text: &str) -> Result<KnowledgeExtraction, AppError> {
        let client = self.get_client(); 

        let agent = client.agent(&self.config.model_name)
            .preamble("You are an expert Ontology Engineer. Extract entities and relationships from the text. \
                       Return strictly JSON format matching this structure: \
                       { \"entities\": [{\"name\": \"...\", \"category\": \"...\"}], \"relations\": [{\"source\": \"...\", \"target\": \"...\", \"relation_type\": \"...\"}] }")
            .build();

        let response = agent.prompt(text).await
            .map_err(|e| AppError::AIError(format!("Extraction failed: {}", e)))?;

        let cleaned_json = self.clean_json_response(&response);

        let extraction: KnowledgeExtraction = from_str(&cleaned_json)
            .map_err(|e| AppError::ParseError(format!("Failed to parse JSON: {} - Raw: {}", e, cleaned_json)))?;

        Ok(extraction)
    }

    async fn generate_inference(&self, prompt: &str) -> Result<InferenceResult, AppError> {
        let client = self.get_client();
        let agent = client.agent(&self.config.model_name).build();
        
        let response = agent.prompt(prompt).await
            .map_err(|e| AppError::AIError(format!("Inference failed: {}", e)))?;
            
        let cleaned = self.clean_json_response(&response);
        
        let result: InferenceResult = serde_json::from_str(&cleaned)
            .map_err(|e| AppError::ParseError(format!("JSON Error: {}", e)))?;
            
        Ok(result)
    }
}