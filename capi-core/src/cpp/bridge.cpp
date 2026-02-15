#include "capi-core/src/cpp/bridge.h"
#include "capi-core/src/genai_bridge.rs.h"

namespace genai_bridge {

// Factory functions
std::unique_ptr<LLMPipelineWrapper> create_pipeline(
    rust::Str model_path,
    rust::Str device
) {
    return std::make_unique<LLMPipelineWrapper>(
        std::string(model_path),
        std::string(device)
    );
}

std::unique_ptr<GenerationConfigWrapper> create_generation_config() {
    return std::make_unique<GenerationConfigWrapper>();
}

// Helper to extract metrics from OpenVINO PerfMetrics
PerfMetricsData extract_metrics(const ov::genai::PerfMetrics& metrics) {
    PerfMetricsData data;
    data.load_time = metrics.load_time;
    data.num_input_tokens = metrics.num_input_tokens;
    data.num_generated_tokens = metrics.num_generated_tokens;
    data.ttft_mean = metrics.ttft.mean;
    data.ttft_std = metrics.ttft.std;
    data.throughput_mean = metrics.throughput.mean;
    data.throughput_std = metrics.throughput.std;
    data.generate_duration_mean = metrics.generate_duration.mean;
    data.generate_duration_std = metrics.generate_duration.std;
    return data;
}

// Pipeline methods
rust::String pipeline_generate(
    const LLMPipelineWrapper& pipeline,
    rust::Str prompt,
    const GenerationConfigWrapper& config
) {
    auto result = pipeline.pipeline->generate(
        std::string(prompt),
        config.config
    );
    
    if (result.texts.empty()) {
        return rust::String("");
    }
    return rust::String(result.texts[0]);
}

GenerationResultData pipeline_generate_with_metrics(
    const LLMPipelineWrapper& pipeline,
    rust::Str prompt,
    const GenerationConfigWrapper& config
) {
    auto result = pipeline.pipeline->generate(
        std::string(prompt),
        config.config
    );
    
    GenerationResultData data;
    data.text = result.texts.empty() ? rust::String("") : rust::String(result.texts[0]);
    data.metrics = extract_metrics(result.perf_metrics);
    return data;
}

// Streaming generation with Rust callback
GenerationResultData pipeline_generate_stream(
    const LLMPipelineWrapper& pipeline,
    rust::Str prompt,
    const GenerationConfigWrapper& config,
    StreamerCallback& callback
) {
    // Create C++ lambda that calls Rust object method
    auto streamer = [&callback](std::string token) -> ov::genai::StreamingStatus {
        rust::Slice<const uint8_t> slice(reinterpret_cast<const uint8_t*>(token.data()), token.size());
        bool should_continue = callback.on_token(slice);
        return should_continue 
            ? ov::genai::StreamingStatus::RUNNING 
            : ov::genai::StreamingStatus::STOP;
    };
    
    auto result = pipeline.pipeline->generate(
        std::string(prompt),
        config.config,
        streamer
    );
    
    GenerationResultData data;
    data.text = result.texts.empty() ? rust::String("") : rust::String(result.texts[0]);
    data.metrics = extract_metrics(result.perf_metrics);
    return data;
}

void pipeline_start_chat(LLMPipelineWrapper& pipeline) {
    pipeline.pipeline->start_chat();
}

void pipeline_finish_chat(LLMPipelineWrapper& pipeline) {
    pipeline.pipeline->finish_chat();
}

// Config methods
void config_set_max_new_tokens(GenerationConfigWrapper& config, size_t max_tokens) {
    config.config.max_new_tokens = max_tokens;
}

void config_set_temperature(GenerationConfigWrapper& config, float temperature) {
    config.config.temperature = temperature;
}

void config_set_top_p(GenerationConfigWrapper& config, float top_p) {
    config.config.top_p = top_p;
}

void config_set_top_k(GenerationConfigWrapper& config, size_t top_k) {
    config.config.top_k = top_k;
}

void config_set_do_sample(GenerationConfigWrapper& config, bool do_sample) {
    config.config.do_sample = do_sample;
}

void config_set_stop_strings(GenerationConfigWrapper& config, rust::Vec<rust::String> stop_strings) {
    std::set<std::string> stops;
    for (const auto& s : stop_strings) {
        stops.insert(std::string(s));
    }
    config.config.stop_strings = stops;
}

void config_set_frequency_penalty(GenerationConfigWrapper& config, float frequency_penalty) {
    config.config.frequency_penalty = frequency_penalty;
}

void config_set_presence_penalty(GenerationConfigWrapper& config, float presence_penalty) {
    config.config.presence_penalty = presence_penalty;
}

void config_set_repetition_penalty(GenerationConfigWrapper& config, float repetition_penalty) {
    config.config.repetition_penalty = repetition_penalty;
}

void config_set_rng_seed(GenerationConfigWrapper& config, size_t seed) {
    config.config.rng_seed = seed;
}

void config_set_logprobs(GenerationConfigWrapper& config, size_t logprobs) {
    config.config.logprobs = logprobs;
}

// Tokenizer methods
std::unique_ptr<TokenizerWrapper> pipeline_get_tokenizer(const LLMPipelineWrapper& pipeline) {
    return std::make_unique<TokenizerWrapper>(pipeline.pipeline->get_tokenizer());
}

size_t tokenizer_count_tokens(TokenizerWrapper& tokenizer, rust::Str text) {
    auto inputs = tokenizer.tokenizer.encode(std::string(text));
    return inputs.input_ids.get_size();
}

} // namespace genai_bridge
