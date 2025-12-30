use anyhow::Error;
use async_trait::async_trait;
use gemini_rust::{Gemini, GenerationConfig, TaskType};
use ndarray::Array1;
use ndarray_linalg::Norm;
use tracing::{error, info};

#[async_trait]
pub trait GeminiContentGeneratorLike: Send + Sync {
    /// Generate a textual response.
    async fn generate_content(&self, prompt: &str) -> Result<String, Error>;

    /// Generates a structured response.
    async fn generate_structured_content(
        &self,
        prompt: &str,
        schema: serde_json::Value,
    ) -> Result<serde_json::Value, Error>;
}

#[async_trait]
pub trait GeminiContentEmbedderLike: Send + Sync {
    async fn embed_content(
        &self,
        content: &str,
        task_type: &TaskType,
        embedding_dimension: u16,
    ) -> Result<Vec<f32>, Error>;
}

#[async_trait]
impl GeminiContentGeneratorLike for Gemini {
    async fn generate_content(&self, prompt: &str) -> Result<String, Error> {
        let response = self
            .generate_content()
            .with_user_message(prompt)
            .execute()
            .await?;

        Ok(response.text())
    }

    async fn generate_structured_content(
        &self,
        prompt: &str,
        schema: serde_json::Value,
    ) -> Result<serde_json::Value, Error> {
        let response = self
            .generate_content()
            .with_user_message(prompt)
            .with_generation_config(GenerationConfig {
                max_output_tokens: Some(8192),
                candidate_count: Some(1),

                ..Default::default()
            })
            .with_response_schema(schema)
            .with_response_mime_type("application/json")
            .execute()
            .await?;

        if let Some(usage_metadata) = &response.usage_metadata {
            info!("response metadata: {:?}", usage_metadata);
        }

        let json_response: serde_json::Value = serde_json::from_str(&response.text())?;

        Ok(json_response)
    }
}

#[async_trait]
impl GeminiContentEmbedderLike for Gemini {
    async fn embed_content(
        &self,
        content: &str,
        task_type: &TaskType,
        embedding_dimension: u16,
    ) -> Result<Vec<f32>, Error> {
        let response = self
            .embed_content()
            .with_text(content)
            .with_task_type(task_type.to_owned())
            .with_output_dimensionality(i32::from(embedding_dimension))
            .execute()
            .await?;

        let embeddings = Array1::from(response.embedding.values);

        let norm = embeddings.norm_l2();

        if norm.abs() <= 0.0001 {
            error!("The input vector is a zero vector and cannot be normalized.");
            return Ok(embeddings.to_vec());
        }
        let normed_embedding = &embeddings / norm;
        Ok(normed_embedding.to_vec())
    }
}
