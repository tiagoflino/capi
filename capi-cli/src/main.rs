use clap::{Parser, Subcommand};
use anyhow::Result;
use std::sync::Arc;
use dialoguer::{Select, Confirm, theme::ColorfulTheme};

#[derive(Parser)]
#[command(name = "capi")]
#[command(about = "Local LLM inference with OpenVINO", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the HTTP server
    Serve,
    /// Model management commands
    Model {
        #[command(subcommand)]
        action: ModelCommands,
    },
    /// Start interactive chat with a model
    Run {
        /// Model ID or name
        model: String,
    },
    /// Generate single response (non-interactive)
    Generate {
        /// The prompt to generate from
        prompt: String,
    },
    /// Benchmark model performance
    Benchmark {
        /// Model ID or name
        model: String,
        /// Number of test runs
        #[arg(long, default_value = "3")]
        runs: usize,
    },
    /// Interactive model browser
    Browse {
        /// Search query
        query: String,
    },
    /// Show or edit configuration
    Config,
    /// Show hardware information
    Hardware,
}

#[derive(Subcommand)]
enum ModelCommands {
    /// List installed models
    List,
    /// Show model information (size, parameters, files)
    Info {
        /// Model identifier (e.g., "microsoft/Phi-3-mini-4k-instruct")
        model: String,
    },
    /// Download model from HuggingFace
    Download {
        /// Model identifier (e.g., "microsoft/Phi-3-mini-4k-instruct-gguf")
        model: String,
        /// Optional display name for the model
        #[arg(long)]
        name: Option<String>,
    },
    /// Register a downloaded model
    Register {
        /// Model identifier (folder name in models directory)
        model: String,
        /// Optional display name for the model
        #[arg(long)]
        name: Option<String>,
    },
    /// Remove a model
    Remove {
        /// Model ID to remove
        model_id: String,
    },
    /// Search HuggingFace models
    Search {
        /// Search query
        query: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Serve => {
            println!("Starting server (use capi-server binary directly instead)");
        }
        Commands::Model { action } => match action {
            ModelCommands::List => {
                let config = capi_core::Config::load()?;
                let db = Arc::new(capi_core::Database::open(config.database_path())?);
                let registry = capi_core::Registry::new(db);

                let models = registry.list_models()?;

                if models.is_empty() {
                    println!("No models installed");
                    println!("\nUse 'capi model search <query>' to find models");
                    println!("Use 'capi model install-from-hf <id>' to install");
                } else {
                    println!("Installed models ({}):\n", models.len());
                    for (idx, model) in models.iter().enumerate() {
                        let quant = model.quantization.as_deref().unwrap_or("-");

                        let size_str = model.size_bytes
                            .map(|s| {
                                if s > 1_000_000_000 {
                                    format!("{:.1}GB", s as f64 / 1_000_000_000.0)
                                } else {
                                    format!("{}MB", s / 1_000_000)
                                }
                            })
                            .unwrap_or_else(|| "-".to_string());

                        let last_used = model.last_used
                            .map(|ts| format_timestamp(ts))
                            .unwrap_or_else(|| "never".to_string());

                        println!("  [{:2}] {:<40} {:>6}  {:>6}  {}",
                            idx + 1,
                            model.name,
                            quant,
                            size_str,
                            last_used
                        );
                    }
                }
            }
            ModelCommands::Info { model } => {
                println!("Fetching model info for: {}", model);
                let downloader = capi_core::Downloader::new();

                match downloader.fetch_model_data(&model).await {
                    Ok(data) => {
                        println!("\nModel: {}", model);

                        if data.has_gguf {
                            println!("  Format: GGUF ✓");

                            if let Some(size) = data.size_bytes {
                                println!("  Size: {:.1} GB", size as f64 / 1_000_000_000.0);
                            }
                            if let Some(arch) = data.architecture {
                                println!("  Architecture: {}", arch);
                            }
                            if let Some(ctx) = data.context_length {
                                println!("  Context length: {}k", ctx / 1000);
                            }

                            let gguf_files: Vec<_> = data.files.iter()
                                .filter(|f| f.ends_with(".gguf"))
                                .collect();

                            println!("  GGUF files: {}", gguf_files.len());
                            for file in gguf_files.iter().take(5) {
                                println!("    - {}", file);
                            }
                        } else {
                            println!("  Format: Not GGUF");
                            println!("  Capi currently supports GGUF models only");
                        }
                    }
                    Err(e) => eprintln!("Error: {}", e),
                }
            }
            ModelCommands::Download { model, name } => {
                println!("Downloading model: {}", model);
                let config = capi_core::Config::load()?;
                let downloader = capi_core::Downloader::new();
                let safe_name = model.replace("/", "_");
                let model_path = config.models_dir.join(&safe_name);

                downloader.download_model(&model, &model_path).await?;
                println!("\n✓ Downloaded to: {}", model_path.display());

                println!("\nRegistering model...");

                let model_file = detect_model_format(&model_path)?;
                let display_name = name.unwrap_or_else(|| model.split('/').last().unwrap_or(&model).to_string());

                let db = Arc::new(capi_core::Database::open(config.database_path())?);
                let registry = capi_core::Registry::new(db);

                let timestamp = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)?
                    .as_secs() as i64;

                let model_record = capi_core::db::ModelRecord {
                    id: safe_name.clone(),
                    name: display_name,
                    path: model_file.to_string_lossy().to_string(),
                    size_bytes: None,
                    quantization: None,
                    context_length: None,
                    created_at: timestamp,
                    last_used: None,
                };

                registry.add_model(model_record)?;
                println!("✓ Model registered: {}", safe_name);
                println!("\nYou can now use this model with:");
                println!("  capi run {}", safe_name);
            }
            ModelCommands::Register { model, name } => {
                let config = capi_core::Config::load()?;
                let db = Arc::new(capi_core::Database::open(config.database_path())?);
                let registry = capi_core::Registry::new(db);

                let model_path = config.models_dir.join(&model);

                if !model_path.exists() {
                    return Err(anyhow::anyhow!("Model path not found: {}", model_path.display()));
                }

                let model_file = detect_model_format(&model_path)?;
                let display_name = name.unwrap_or_else(|| model.clone());

                let timestamp = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)?
                    .as_secs() as i64;

                let model_record = capi_core::db::ModelRecord {
                    id: model.clone(),
                    name: display_name,
                    path: model_file.to_string_lossy().to_string(),
                    size_bytes: None,
                    quantization: None,
                    context_length: None,
                    created_at: timestamp,
                    last_used: None,
                };

                registry.add_model(model_record)?;
                println!("✓ Model registered: {}", model);
                println!("\nYou can now use this model with:");
                println!("  capi run {}", model);
            }
            ModelCommands::Remove { model_id } => {
                let config = capi_core::Config::load()?;
                let db = Arc::new(capi_core::Database::open(config.database_path())?);
                let registry = capi_core::Registry::new(db);

                registry.remove_model(&model_id)?;
                println!("Removed model: {}", model_id);
            }
            ModelCommands::Search { query } => {
                let downloader = capi_core::Downloader::new();
                let results = downloader.search_models(&query).await?;

                if results.is_empty() {
                    println!("No models found for '{}'", query);
                } else {
                    println!("Found {} models:\n", results.len());
                    println!("  ID                                                    Downloads");
                    println!("  ─────────────────────────────────────────────────────────────────");

                    for (idx, model) in results.iter().enumerate() {
                        let downloads_str = if model.downloads >= 1_000_000 {
                            format!("{:.1}M", model.downloads as f64 / 1_000_000.0)
                        } else if model.downloads >= 1_000 {
                            format!("{:.0}K", model.downloads as f64 / 1_000.0)
                        } else {
                            model.downloads.to_string()
                        };

                        println!("  [{:2}] {:<47} ↓{:>6}",
                            idx + 1,
                            model.id,
                            downloads_str
                        );
                    }
                    println!("\n↓ = Downloads  |  Use 'capi model info <id>' for size/arch/context");
                    println!("Use 'capi model download <id>' to download & register");
                }
            }
        },
        Commands::Run { model } => {
            use std::io::{self, Write};

            let config = capi_core::Config::load()?;
            let db = Arc::new(capi_core::Database::open(config.database_path())?);
            let registry = capi_core::Registry::new(db);

            let model_record = registry.get_model(&model)?
                .or_else(|| {
                    registry.list_models()
                        .ok()
                        .and_then(|models| {
                            models.into_iter()
                                .find(|m| m.name.contains(&model) || m.id.contains(&model))
                        })
                })
                .ok_or_else(|| anyhow::anyhow!("Model not found: {}", model))?;

            println!("Loading {}...", model_record.name);
            println!("Model path: {}", model_record.path);

            let model_path = std::path::Path::new(&model_record.path);

            if !model_path.exists() {
                return Err(anyhow::anyhow!("Model file not found at: {}", model_record.path));
            }

            let devices = capi_core::detect_devices()?;
            let device = capi_core::select_best_device(&devices, &config.device_preference)
                .unwrap_or_else(|| "CPU".to_string());

            println!("Loading on device: {}", device);

            let mut session = capi_core::InferenceSession::load(model_path, &device)?;

            println!("Ready! Type your message (or /exit to quit)\n");

            let stdin = io::stdin();
            let mut conversation = Vec::new();

            loop {
                print!(">>> ");
                io::stdout().flush()?;

                let mut input = String::new();
                stdin.read_line(&mut input)?;

                let input = input.trim();

                if input == "/exit" || input == "/quit" {
                    println!("Goodbye!");
                    break;
                }

                if input.is_empty() {
                    continue;
                }

                conversation.push(format!("User: {}", input));

                let full_prompt = conversation.join("\n") + "\nAssistant:";

                let (output, metrics) = session.generate_with_metrics(&full_prompt, 100)?;
                println!("{}", output);
                println!("({:.1} tok/s, {} tokens)\n", metrics.tokens_per_second, metrics.num_output_tokens);

                conversation.push(format!("Assistant: {}", output));
            }
        }
        Commands::Generate { prompt } => {
            let config = capi_core::Config::load()?;
            let db = Arc::new(capi_core::Database::open(config.database_path())?);
            let registry = capi_core::Registry::new(db);

            let models = registry.list_models()?;
            if models.is_empty() {
                eprintln!("No models installed");
                eprintln!("Use 'capi model install-from-hf <id>' to install a model");
                return Ok(());
            }

            let active_model = models.first().unwrap();
            let model_path = std::path::Path::new(&active_model.path);

            let devices = capi_core::detect_devices()?;
            let device = capi_core::select_best_device(&devices, &config.device_preference)
                .unwrap_or_else(|| "CPU".to_string());

            let mut session = capi_core::InferenceSession::load(model_path, &device)?;
            let output = session.generate(&prompt, 50)?;

            println!("{}", output);
        }
        Commands::Benchmark { model, runs } => {
            let config = capi_core::Config::load()?;
            let db = Arc::new(capi_core::Database::open(config.database_path())?);
            let registry = capi_core::Registry::new(db);

            let model_record = registry.get_model(&model)?
                .or_else(|| {
                    registry.list_models()
                        .ok()
                        .and_then(|models| {
                            models.into_iter()
                                .find(|m| m.name.contains(&model) || m.id.contains(&model))
                        })
                })
                .ok_or_else(|| anyhow::anyhow!("Model not found: {}", model))?;

            println!("Benchmarking: {}\n", model_record.name);

            let model_path = std::path::Path::new(&model_record.path);
            let devices = capi_core::detect_devices()?;
            let device = capi_core::select_best_device(&devices, &config.device_preference)
                .unwrap_or_else(|| "CPU".to_string());

            println!("Loading model on {}...", device);
            let session = capi_core::InferenceSession::load(model_path, &device)?;

            let test_prompts = vec![
                "Hello, how are you?",
                "What is the capital of France?",
                "Tell me a short story.",
            ];

            let mut all_tps = Vec::new();
            let mut all_ttft = Vec::new();

            for run in 0..runs {
                println!("\nRun {}/{}:", run + 1, runs);

                for prompt in &test_prompts {
                    let (_, metrics) = session.generate_with_metrics(prompt, 50)?;

                    println!("  Prompt: {}", prompt);
                    println!("    Tokens/sec: {:.2}", metrics.tokens_per_second);
                    println!("    TTFT: {:.2} ms", metrics.time_to_first_token_ms);
                    println!("    Input tokens: {}", metrics.num_input_tokens);
                    println!("    Output tokens: {}", metrics.num_output_tokens);

                    all_tps.push(metrics.tokens_per_second);
                    all_ttft.push(metrics.time_to_first_token_ms);
                }
            }

            let avg_tps: f32 = all_tps.iter().sum::<f32>() / all_tps.len() as f32;
            let avg_ttft: f32 = all_ttft.iter().sum::<f32>() / all_ttft.len() as f32;

            println!("\n=== Summary ===");
            println!("Average throughput: {:.2} tokens/sec", avg_tps);
            println!("Average TTFT: {:.2} ms", avg_ttft);
            println!("Device: {}", device);
        }
        Commands::Browse { query } => {
            let downloader = capi_core::Downloader::new();
            let models = downloader.search_models(&query).await?;

            if models.is_empty() {
                println!("No models found for '{}'", query);
                return Ok(());
            }

            let items: Vec<String> = models.iter()
                .map(|m| {
                    let downloads_str = if m.downloads >= 1_000_000 {
                        format!("{:.1}M", m.downloads as f64 / 1_000_000.0)
                    } else if m.downloads >= 1_000 {
                        format!("{:.0}K", m.downloads as f64 / 1_000.0)
                    } else {
                        m.downloads.to_string()
                    };
                    format!("{} (↓{})", m.id, downloads_str)
                })
                .collect();

            let selection = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("Select a model")
                .items(&items)
                .interact()?;

            let selected = &models[selection];

            println!("\nSearching for quantized versions...");
            let quantized = downloader.find_quantized_versions(&selected.id).await?;

            let model_to_use = if quantized.is_empty() {
                println!("No quantized versions found, using base model");
                selected.clone()
            } else {
                println!("Found {} quantized version(s):\n", quantized.len());

                let quant_items: Vec<String> = quantized.iter()
                    .map(|m| {
                        let downloads_str = if m.downloads >= 1_000_000 {
                            format!("{:.1}M", m.downloads as f64 / 1_000_000.0)
                        } else if m.downloads >= 1_000 {
                            format!("{:.0}K", m.downloads as f64 / 1_000.0)
                        } else {
                            m.downloads.to_string()
                        };

                        let quant_info = extract_quant_info(&m.name);

                        if let Some(info) = quant_info {
                            format!("{} [{}] (↓{})", m.id, info, downloads_str)
                        } else {
                            format!("{} (↓{})", m.id, downloads_str)
                        }
                    })
                    .collect();

                let quant_selection = Select::with_theme(&ColorfulTheme::default())
                    .with_prompt("Select quantized version")
                    .items(&quant_items)
                    .interact()?;

                quantized[quant_selection].clone()
            };

            println!("\nFetching details for {}...", model_to_use.id);
            let model_data = downloader.fetch_model_data(&model_to_use.id).await?;

            println!("\n{}", model_to_use.id);
            if let Some(size) = model_data.size_bytes {
                println!("Size: {:.1} GB", size as f64 / 1_000_000_000.0);
            }
            if let Some(arch) = &model_data.architecture {
                println!("Architecture: {}", arch);
            }
            if let Some(ctx) = model_data.context_length {
                println!("Context: {}k", ctx / 1000);
            }

            let gguf_files: Vec<&capi_core::FileInfo> = model_data.files_with_size.iter()
                .filter(|f| f.name.ends_with(".gguf"))
                .collect();

            if gguf_files.is_empty() {
                println!("\nThis model has no GGUF files.");
                println!("Capi currently supports GGUF models only.");
                return Ok(());
            }

            println!("\nAvailable quantizations:");

            let quant_items: Vec<String> = gguf_files.iter()
                .map(|f| {
                    let size_str = if let Some(size) = f.size {
                        if size > 1_000_000_000 {
                            format!("{:.1} GB", size as f64 / 1_000_000_000.0)
                        } else {
                            format!("{} MB", size / 1_000_000)
                        }
                    } else {
                        "?".to_string()
                    };

                    if let Some(quant) = extract_quantization(&f.name) {
                        format!("{:<10} {:>8}  {}", quant, size_str, f.name)
                    } else {
                        format!("{:>19}  {}", size_str, f.name)
                    }
                })
                .collect();

            let quant_selection = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("Select quantization")
                .items(&quant_items)
                .interact()?;

            let selected_file = &gguf_files[quant_selection].name;

            if !Confirm::with_theme(&ColorfulTheme::default())
                .with_prompt(format!("Download {}?", selected_file))
                .interact()? {
                println!("Cancelled");
                return Ok(());
            }

            let config = capi_core::Config::load()?;
            let safe_name = model_to_use.id.replace("/", "_");
            let model_path = config.models_dir.join(&safe_name);

            std::fs::create_dir_all(&model_path)?;

            println!("\nDownloading {}...", selected_file);

            let url = format!("https://huggingface.co/{}/resolve/main/{}", model_to_use.id, selected_file);

            downloader.download_file_with_progress(&url, &model_path.join(selected_file), |current, total| {
                if total > 0 {
                    let pct = (current as f64 / total as f64 * 100.0) as u32;
                    let mb_current = current as f64 / 1_000_000.0;
                    let mb_total = total as f64 / 1_000_000.0;

                    let bar_width = 40;
                    let filled = (bar_width * pct as usize) / 100;
                    let empty = bar_width - filled;
                    let bar = format!("[{}{}]", "=".repeat(filled), " ".repeat(empty));

                    print!("\r  {} {:.1}/{:.1} MB ({}%)  ", bar, mb_current, mb_total, pct);
                    use std::io::Write;
                    std::io::stdout().flush().ok();
                }
            }).await?;

            println!("\n✓ Downloaded");

            println!("\nRegistering model...");
            let model_file = model_path.join(selected_file);

            let db = Arc::new(capi_core::Database::open(config.database_path())?);
            let registry = capi_core::Registry::new(db);

            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs() as i64;

            let file_size = gguf_files[quant_selection].size.map(|s| s as i64);

            let model_record = capi_core::db::ModelRecord {
                id: safe_name.clone(),
                name: model_to_use.id.split('/').last().unwrap_or(&model_to_use.id).to_string(),
                path: model_file.to_string_lossy().to_string(),
                size_bytes: file_size,
                quantization: Some(selected_file.to_string()),
                context_length: model_data.context_length.map(|c| c as i64),
                created_at: timestamp,
                last_used: None,
            };

            registry.add_model(model_record)?;
            println!("✓ Model registered: {}", safe_name);
            println!("\nYou can now use: capi run {}", safe_name);
        }
        Commands::Config => {
            let config = capi_core::Config::load()?;
            println!("Configuration:");
            println!("  Server: {}:{}", config.server_host, config.server_port);
            println!("  Server URL: {}", config.server_url());
            println!("  Models directory: {}", config.models_dir.display());
            println!("  Data directory: {}", config.data_dir.display());
            println!("  Device preference: {:?}", config.device_preference);
            println!("  Auto start: {}", config.auto_start);
            println!("  Keep server running: {}", config.keep_server_running);
        }
        Commands::Hardware => {
            println!("Detecting hardware...");
            let devices = capi_core::detect_devices()?;

            println!("Available devices:");
            for device in &devices {
                println!("  {} - {:?} (available: {})",
                    device.name, device.device_type, device.available);
            }

            let config = capi_core::Config::load()?;
            if let Some(selected) = capi_core::select_best_device(&devices, &config.device_preference) {
                println!("\nSelected device (based on {:?} preference): {}",
                    config.device_preference, selected);
            }
        }
    }

    Ok(())
}

fn detect_model_format(model_path: &std::path::Path) -> Result<std::path::PathBuf> {
    if let Some(gguf) = find_file_with_extension(model_path, "gguf")? {
        Ok(gguf)
    } else if model_path.join("openvino_model.xml").exists() {
        Ok(model_path.join("openvino_model.xml"))
    } else {
        Err(anyhow::anyhow!(
            "No supported model format found. Expected:\n\
             - GGUF file (.gguf)\n\
             - OpenVINO IR (openvino_model.xml)\n\n\
             Capi currently supports GGUF models.\n\
             Download GGUF models from HuggingFace (look for -gguf or -GGUF suffix)."
        ))
    }
}

fn find_file_with_extension(dir: &std::path::Path, ext: &str) -> Result<Option<std::path::PathBuf>> {
    Ok(std::fs::read_dir(dir)?
        .filter_map(|e| e.ok())
        .find(|e| e.path().extension().map_or(false, |e| e == ext))
        .map(|e| e.path()))
}

fn format_timestamp(ts: i64) -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    let diff = now - ts;

    if diff < 60 {
        "just now".to_string()
    } else if diff < 3600 {
        format!("{}m ago", diff / 60)
    } else if diff < 86400 {
        format!("{}h ago", diff / 3600)
    } else if diff < 604800 {
        format!("{}d ago", diff / 86400)
    } else {
        "over a week ago".to_string()
    }
}

fn extract_quantization(filename: &str) -> Option<String> {
    let patterns = vec![
        "Q2_K", "Q3_K_S", "Q3_K_M", "Q3_K_L",
        "Q4_0", "Q4_1", "Q4_K_S", "Q4_K_M",
        "Q5_0", "Q5_1", "Q5_K_S", "Q5_K_M",
        "Q6_K", "Q8_0",
        "IQ1_S", "IQ1_M", "IQ2_XXS", "IQ2_XS", "IQ2_S", "IQ2_M",
        "IQ3_XXS", "IQ3_XS", "IQ3_S", "IQ3_M",
        "IQ4_XS", "IQ4_NL",
        "F16", "F32",
    ];

    let upper = filename.to_uppercase();

    for pattern in patterns {
        if upper.contains(pattern) {
            return Some(pattern.to_string());
        }
    }

    None
}

fn extract_quant_info(model_name: &str) -> Option<String> {
    let upper = model_name.to_uppercase();
    let mut quants = Vec::new();

    let patterns = vec![
        "Q2_K", "Q3_K_S", "Q3_K_M", "Q3_K_L",
        "Q4_0", "Q4_1", "Q4_K_S", "Q4_K_M",
        "Q5_0", "Q5_1", "Q5_K_S", "Q5_K_M",
        "Q6_K", "Q8_0",
        "IQ1_S", "IQ1_M", "IQ2_XXS", "IQ2_XS", "IQ2_S", "IQ2_M",
        "IQ3_XXS", "IQ3_XS", "IQ3_S", "IQ3_M",
        "IQ4_XS", "IQ4_NL",
        "F16", "F32",
    ];

    for pattern in &patterns {
        if upper.contains(pattern) {
            quants.push(pattern.to_string());
        }
    }

    if quants.is_empty() {
        None
    } else if quants.len() == 1 {
        Some(quants[0].clone())
    } else {
        Some(format!("{} variants", quants.len()))
    }
}
