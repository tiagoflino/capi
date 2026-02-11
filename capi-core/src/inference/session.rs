use super::genai::LLMPipeline;
use anyhow::Result;
use std::path::Path;
use crate::hardware::{detect_system_resources, validate_model_load, ValidationResult};
use crate::config::Config;
use crate::model_manager::ModelLock;

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
    _lock: Option<ModelLock>,
    context_tokens: usize,
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

        // Validate resources before loading
        if let Ok(file_size) = std::fs::metadata(path_to_use).map(|m| m.len()) {
            let estimated_memory = (file_size as f64 * 1.5) as u64;
            let config = Config::load()?;

            if let Ok(resources) = detect_system_resources() {
                match validate_model_load(estimated_memory, device, &resources, &config.resource_mode)? {
                    ValidationResult::Sufficient => {},
                    ValidationResult::Warning { message } => {
                        eprintln!("\nWarning: {}", message);
                        eprintln!("This may cause OOM errors or system instability.");

                        if matches!(config.resource_mode, crate::hardware::ResourceMode::Loose) {
                            eprintln!("Warning: Resource mode is Loose. Continuing despite potential memory pressure.");
                        }
                    },
                    ValidationResult::Insufficient { message } => {
                        return Err(anyhow::anyhow!(
                            "Insufficient memory to load model\n\n{}\n\nSuggestions:\n\
                            - Close other applications to free memory\n\
                            - Try a more quantized version (Q3_K_M, Q2_K)\n\
                            - Use a smaller model\n\
                            - Change resource mode: capi config set-resource-mode loose",
                            message
                        ));
                    },
                }
            }
        }

        let pipeline = LLMPipeline::new(
            path_to_use.to_str().unwrap(),
            device,
        ).map_err(|e| anyhow::anyhow!("Failed to create pipeline: {}", e))?;

        Ok(Self {
            pipeline,
            in_chat_mode: false,
            _lock: None,
            context_tokens: 0,
        })
    }

    pub fn load_with_lock(model_path: &Path, device: &str, model_id: &str) -> Result<Self> {
        let lock = ModelLock::try_acquire(model_id)?;

        let path_to_use = if model_path.extension().and_then(|e| e.to_str()) == Some("gguf") {
            model_path
        } else if model_path.is_dir() {
            model_path
        } else {
            model_path.parent()
                .ok_or_else(|| anyhow::anyhow!("Invalid model path"))?
        };

        if let Ok(file_size) = std::fs::metadata(path_to_use).map(|m| m.len()) {
            let estimated_memory = (file_size as f64 * 1.5) as u64;
            let config = Config::load()?;

            if let Ok(resources) = detect_system_resources() {
                match validate_model_load(estimated_memory, device, &resources, &config.resource_mode)? {
                    ValidationResult::Sufficient => {},
                    ValidationResult::Warning { message } => {
                        eprintln!("\nWarning: {}", message);
                        eprintln!("This may cause OOM errors or system instability.");

                        if matches!(config.resource_mode, crate::hardware::ResourceMode::Loose) {
                            eprintln!("Warning: Resource mode is Loose. Continuing despite potential memory pressure.");
                        }
                    },
                    ValidationResult::Insufficient { message } => {
                        return Err(anyhow::anyhow!(
                            "Insufficient memory to load model\n\n{}\n\nSuggestions:\n\
                            - Close other applications to free memory\n\
                            - Try a more quantized version (Q3_K_M, Q2_K)\n\
                            - Use a smaller model\n\
                            - Change resource mode: capi config set-resource-mode loose",
                            message
                        ));
                    },
                }
            }
        }

        let pipeline = LLMPipeline::new(
            path_to_use.to_str().unwrap(),
            device,
        ).map_err(|e| {
            anyhow::anyhow!("Failed to create pipeline: {}", e)
        })?;

        Ok(Self {
            pipeline,
            in_chat_mode: false,
            _lock: Some(lock),
            context_tokens: 0,
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

    pub fn generate(&mut self, prompt: &str, max_tokens: usize) -> Result<String> {
        let text = self.pipeline.generate(prompt, max_tokens)
            .map_err(|e| anyhow::anyhow!("Generation failed: {}", e))?;
        
        self.context_tokens = self.pipeline.count_tokens(prompt) + self.pipeline.count_tokens(&text);
        Ok(text)
    }

    pub fn generate_with_metrics(&mut self, prompt: &str, max_tokens: usize) -> Result<(String, InferenceMetrics)> {
        let result = self.pipeline.generate_with_metrics(prompt, max_tokens)
            .map_err(|e| anyhow::anyhow!("Generation failed: {}", e))?;

        let (throughput, _) = result.metrics.throughput();
        let (ttft, _) = result.metrics.ttft();
        let (duration, _) = result.metrics.generate_duration();

        let num_input = result.metrics.num_input_tokens();
        let num_output = result.metrics.num_generated_tokens();

        self.context_tokens = num_input + num_output;

        let metrics = InferenceMetrics {
            tokens_per_second: throughput,
            time_to_first_token_ms: ttft,
            num_input_tokens: num_input,
            num_output_tokens: num_output,
            total_time_ms: duration,
        };

        Ok((result.text, metrics))
    }

    pub fn generate_stream<F>(&mut self, prompt: &str, max_tokens: usize, mut callback: F) -> Result<(String, InferenceMetrics)> 
    where F: FnMut(&str) -> bool
    {
        // Estimate prompt tokens if possible
        let _prompt_tokens = self.pipeline.count_tokens(prompt);
        
        let result = self.pipeline.generate_stream(prompt, max_tokens, |token| {
            callback(token)
        })?;

        let (throughput, _) = result.metrics.throughput();
        let (ttft, _) = result.metrics.ttft();
        let (duration, _) = result.metrics.generate_duration();

        let num_input = result.metrics.num_input_tokens();
        let num_output = result.metrics.num_generated_tokens();

        // Update session context size
        // Note: OpenVINO GenAI in chat mode handles history, 
        // but we want to know the total "active" context.
        self.context_tokens = num_input + num_output;

        let metrics = InferenceMetrics {
            tokens_per_second: throughput,
            time_to_first_token_ms: ttft,
            num_input_tokens: num_input,
            num_output_tokens: num_output,
            total_time_ms: duration,
        };

        Ok((result.text, metrics))
    }

    pub fn get_context_tokens(&self) -> usize {
        self.context_tokens
    }
}
