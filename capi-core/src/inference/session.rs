use super::genai::LLMPipeline;
use anyhow::Result;
use std::path::Path;

pub struct InferenceMetrics {
    pub tokens_per_second: f32,
    pub time_to_first_token_ms: f32,
    pub num_input_tokens: usize,
    pub num_output_tokens: usize,
    pub total_time_ms: f32,
}

pub struct InferenceSession {
    pipeline: LLMPipeline,
    in_chat_mode: bool,
}

impl InferenceSession {
    pub fn load(model_path: &Path, device: &str) -> Result<Self> {
        let path_to_use = if model_path.extension().and_then(|e| e.to_str()) == Some("gguf") {
            model_path
        } else if model_path.is_dir() {
            model_path
        } else {
            model_path.parent()
                .ok_or_else(|| anyhow::anyhow!("Invalid model path"))?
        };

        let pipeline = LLMPipeline::new(
            path_to_use.to_str().unwrap(),
            device,
        ).map_err(|e| anyhow::anyhow!("Failed to create pipeline: {}", e))?;

        Ok(Self {
            pipeline,
            in_chat_mode: false,
        })
    }

    pub fn start_chat(&mut self) -> Result<()> {
        self.pipeline.start_chat()
            .map_err(|e| anyhow::anyhow!("Failed to start chat: {}", e))?;
        self.in_chat_mode = true;
        Ok(())
    }

    pub fn finish_chat(&mut self) -> Result<()> {
        self.pipeline.finish_chat()
            .map_err(|e| anyhow::anyhow!("Failed to finish chat: {}", e))?;
        self.in_chat_mode = false;
        Ok(())
    }

    pub fn generate(&self, prompt: &str, max_tokens: usize) -> Result<String> {
        self.pipeline.generate(prompt, max_tokens)
            .map_err(|e| anyhow::anyhow!("Generation failed: {}", e))
    }

    pub fn generate_with_metrics(&self, prompt: &str, max_tokens: usize) -> Result<(String, InferenceMetrics)> {
        let result = self.pipeline.generate_with_metrics(prompt, max_tokens)
            .map_err(|e| anyhow::anyhow!("Generation failed: {}", e))?;

        let (throughput, _) = result.metrics.throughput()
            .map_err(|e| anyhow::anyhow!("Failed to get throughput: {}", e))?;
        let (ttft, _) = result.metrics.ttft()
            .map_err(|e| anyhow::anyhow!("Failed to get TTFT: {}", e))?;
        let (duration, _) = result.metrics.generate_duration()
            .map_err(|e| anyhow::anyhow!("Failed to get duration: {}", e))?;

        let metrics = InferenceMetrics {
            tokens_per_second: throughput,
            time_to_first_token_ms: ttft,
            num_input_tokens: result.metrics.num_input_tokens()
                .map_err(|e| anyhow::anyhow!("Failed to get input tokens: {}", e))?,
            num_output_tokens: result.metrics.num_generated_tokens()
                .map_err(|e| anyhow::anyhow!("Failed to get output tokens: {}", e))?,
            total_time_ms: duration,
        };

        Ok((result.text, metrics))
    }
}
