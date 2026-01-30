#pragma once

#include <memory>
#include <string>
#include <vector>
#include <cstdint>
#include <functional>
#include <set>

#include "rust/cxx.h"
#include <openvino/genai/llm_pipeline.hpp>
#include <openvino/genai/generation_config.hpp>
#include <openvino/genai/perf_metrics.hpp>

namespace genai_bridge {

// Opaque wrapper types for Rust - must be fully defined in header for cxx
struct LLMPipelineWrapper {
    std::unique_ptr<ov::genai::LLMPipeline> pipeline;
    
    LLMPipelineWrapper(const std::string& model_path, const std::string& device)
        : pipeline(std::make_unique<ov::genai::LLMPipeline>(model_path, device)) {}
};

struct GenerationConfigWrapper {
    ov::genai::GenerationConfig config;
};

struct TokenizerWrapper {
    ov::genai::Tokenizer tokenizer;
    TokenizerWrapper(ov::genai::Tokenizer t) : tokenizer(std::move(t)) {}
};

// Shared data struct declarations - these are defined by cxx in the generated code
struct PerfMetricsData;
struct GenerationResultData;

// Forward declaration of Rust type
struct StreamerCallback;

// Factory functions
std::unique_ptr<LLMPipelineWrapper> create_pipeline(
    rust::Str model_path,
    rust::Str device
);

std::unique_ptr<GenerationConfigWrapper> create_generation_config();

// Tokenizer methods
std::unique_ptr<TokenizerWrapper> pipeline_get_tokenizer(const LLMPipelineWrapper& pipeline);
size_t tokenizer_count_tokens(TokenizerWrapper& tokenizer, rust::Str text);

// Pipeline methods
rust::String pipeline_generate(
    const LLMPipelineWrapper& pipeline,
    rust::Str prompt,
    const GenerationConfigWrapper& config
);

GenerationResultData pipeline_generate_with_metrics(
    const LLMPipelineWrapper& pipeline,
    rust::Str prompt,
    const GenerationConfigWrapper& config
);

GenerationResultData pipeline_generate_stream(
    const LLMPipelineWrapper& pipeline,
    rust::Str prompt,
    const GenerationConfigWrapper& config,
    StreamerCallback& callback
);

void pipeline_start_chat(LLMPipelineWrapper& pipeline);
void pipeline_finish_chat(LLMPipelineWrapper& pipeline);

// Config methods
void config_set_max_new_tokens(GenerationConfigWrapper& config, size_t max_tokens);
void config_set_temperature(GenerationConfigWrapper& config, float temperature);
void config_set_top_p(GenerationConfigWrapper& config, float top_p);
void config_set_top_k(GenerationConfigWrapper& config, size_t top_k);
void config_set_do_sample(GenerationConfigWrapper& config, bool do_sample);
void config_set_stop_strings(GenerationConfigWrapper& config, rust::Vec<rust::String> stop_strings);

} // namespace genai_bridge
