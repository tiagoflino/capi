use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::fs::{self, File};
use tokio::io::AsyncWriteExt;

#[derive(Debug)]
pub struct ModelInfo {
    pub size_bytes: Option<u64>,
    pub num_parameters: Option<String>,
    pub files: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct FileInfo {
    pub name: String,
    pub size: Option<u64>,
}

#[derive(Debug)]
pub struct ModelData {
    pub has_gguf: bool,
    pub size_bytes: Option<u64>,
    pub architecture: Option<String>,
    pub context_length: Option<u64>,
    pub files: Vec<String>,
    pub files_with_size: Vec<FileInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HuggingFaceModel {
    pub id: String,
    pub author: String,
    pub name: String,
    pub downloads: u64,
    pub likes: u64,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub size_bytes: Option<u64>,
    pub num_parameters: Option<String>,
    pub architecture: Option<String>,
    pub context_length: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct HfApiModel {
    #[serde(rename = "modelId")]
    model_id: Option<String>,
    id: Option<String>,
    author: Option<String>,
    #[serde(rename = "modelName")]
    model_name: Option<String>,
    downloads: Option<u64>,
    likes: Option<u64>,
    #[serde(rename = "cardData")]
    card_data: Option<serde_json::Value>,
    tags: Option<Vec<String>>,
    // safetensors and gguf removed as unused
}

// HfSearchResponse removed as unused

pub struct Downloader {
    client: Client,
}

impl Downloader {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    pub async fn search_models(&self, query: &str) -> Result<Vec<HuggingFaceModel>> {
        let url = format!(
            "https://huggingface.co/api/models?search={}&filter=text-generation&sort=downloads&limit=20",
            urlencoding::encode(query)
        );

        let response = self.client.get(&url).send().await?;
        let models: Vec<HfApiModel> = response.json().await?;

        Ok(models.into_iter()
            .filter(|m| is_supported_model(m))
            .map(|m| {
                let id = m.model_id.or(m.id).unwrap_or_default();
                let parts: Vec<&str> = id.split('/').collect();
                let id_clone = id.clone();

                HuggingFaceModel {
                    id: id_clone,
                    author: m.author.or_else(|| parts.first().map(|s| s.to_string())).unwrap_or_default(),
                    name: m.model_name.or_else(|| parts.get(1).map(|s| s.to_string())).unwrap_or_default(),
                    downloads: m.downloads.unwrap_or(0),
                    likes: m.likes.unwrap_or(0),
                    tags: m.tags.unwrap_or_default(),
                    description: m.card_data.as_ref()
                        .and_then(|c| c.get("description"))
                        .and_then(|d| d.as_str())
                        .map(String::from),
                    size_bytes: None,
                    architecture: None,
                    context_length: None,
                    num_parameters: None,
                }
            })
            .collect())
    }

    pub async fn find_quantized_versions(&self, base_model_id: &str) -> Result<Vec<HuggingFaceModel>> {
        let search_name = base_model_id.split('/').last().unwrap_or(base_model_id);
        let url = format!(
            "https://huggingface.co/api/models?search={}-gguf&filter=text-generation&sort=downloads&limit=100",
            urlencoding::encode(search_name)
        );

        let response = self.client.get(&url).send().await?;
        let models: Vec<HfApiModel> = response.json().await?;

        let tag_to_find = format!("base_model:quantized:{}", base_model_id);

        let results = models.into_iter()
            .filter(|m| {
                if !is_supported_model(m) {
                    return false;
                }

                let tags = m.tags.as_ref().map(|t| t.as_slice()).unwrap_or(&[]);
                tags.iter().any(|t| t == &tag_to_find || t.contains(&base_model_id))
            })
            .map(|m| {
                let id = m.model_id.clone().or(m.id.clone()).unwrap_or_default();
                let parts: Vec<&str> = id.split('/').collect();
                let id_clone = id.clone();

                HuggingFaceModel {
                    id: id_clone,
                    author: m.author.or_else(|| parts.first().map(|s| s.to_string())).unwrap_or_default(),
                    name: m.model_name.or_else(|| parts.get(1).map(|s| s.to_string())).unwrap_or_default(),
                    downloads: m.downloads.unwrap_or(0),
                    likes: m.likes.unwrap_or(0),
                    tags: m.tags.unwrap_or_default(),
                    description: m.card_data.as_ref()
                        .and_then(|c| c.get("description"))
                        .and_then(|d| d.as_str())
                        .map(String::from),
                    size_bytes: None,
                    architecture: None,
                    context_length: None,
                    num_parameters: None,
                }
            })
            .collect();

        Ok(results)
    }

    pub async fn get_model_info(&self, model_id: &str) -> Result<ModelInfo> {
        let url = format!("https://huggingface.co/api/models/{}", model_id);
        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("Model not found: {}", model_id));
        }

        let info: serde_json::Value = response.json().await?;

        let size_bytes = info.get("safetensors")
            .and_then(|s| s.get("total"))
            .and_then(|t| t.as_u64());

        let num_parameters = info.get("safetensors")
            .and_then(|s| s.get("parameters"))
            .and_then(|p| p.as_str())
            .map(String::from);

        let siblings = info.get("siblings")
            .and_then(|s| s.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|f| f.get("rfilename").and_then(|n| n.as_str()).map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        Ok(ModelInfo {
            size_bytes,
            num_parameters,
            files: siblings,
        })
    }

    fn fetch_model_data_legacy(&self, _model_id: &str, info: serde_json::Value) -> Result<ModelData> {
        let siblings: Vec<String> = info.get("siblings")
            .and_then(|s| s.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|f| f.get("rfilename").and_then(|n| n.as_str()).map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        let files_with_size: Vec<FileInfo> = info.get("siblings")
            .and_then(|s| s.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|f| {
                        let name = f.get("rfilename").and_then(|n| n.as_str())?;
                        let size = f.get("size")
                            .and_then(|s| s.as_u64())
                            .or_else(|| f.get("lfs").and_then(|lfs| lfs.get("size")).and_then(|s| s.as_u64()));

                        Some(FileInfo {
                            name: name.to_string(),
                            size,
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();

        let has_gguf = siblings.iter().any(|f| f.ends_with(".gguf"));

        let (size_bytes, architecture, context_length) = if let Some(gguf) = info.get("gguf") {
            (
                gguf.get("total").and_then(|t| t.as_u64()),
                gguf.get("architecture").and_then(|a| a.as_str()).map(String::from),
                gguf.get("context_length").and_then(|c| c.as_u64()),
            )
        } else {
            (None, None, None)
        };

        Ok(ModelData {
            has_gguf,
            size_bytes,
            architecture,
            context_length,
            files: siblings,
            files_with_size,
        })
    }

    pub async fn fetch_model_data(&self, model_id: &str) -> Result<ModelData> {
        let url = format!("https://huggingface.co/api/models/{}/tree/main", model_id);
        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            let url_alt = format!("https://huggingface.co/api/models/{}", model_id);
            let response = self.client.get(&url_alt).send().await?;
            if !response.status().is_success() {
                return Err(anyhow::anyhow!("Model not found: {}", model_id));
            }
            return self.fetch_model_data_legacy(model_id, response.json().await?);
        }

        let files: Vec<serde_json::Value> = response.json().await?;

        let url_info = format!("https://huggingface.co/api/models/{}", model_id);
        let response_info = self.client.get(&url_info).send().await?;
        let info: serde_json::Value = response_info.json().await?;

        let siblings: Vec<String> = info.get("siblings")
            .and_then(|s| s.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|f| f.get("rfilename").and_then(|n| n.as_str()).map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        let files_with_size: Vec<FileInfo> = files.iter()
            .filter_map(|f| {
                let name = f.get("path").and_then(|n| n.as_str())?;
                let size = f.get("size").and_then(|s| s.as_u64());

                Some(FileInfo {
                    name: name.to_string(),
                    size,
                })
            })
            .collect();

        let has_gguf = siblings.iter().any(|f| f.ends_with(".gguf"));

        let (size_bytes, architecture, context_length) = if let Some(gguf) = info.get("gguf") {
            (
                gguf.get("total").and_then(|t| t.as_u64()),
                gguf.get("architecture").and_then(|a| a.as_str()).map(String::from),
                gguf.get("context_length").and_then(|c| c.as_u64()),
            )
        } else {
            (None, None, None)
        };

        Ok(ModelData {
            has_gguf,
            size_bytes,
            architecture,
            context_length,
            files: siblings,
            files_with_size,
        })
    }

    pub async fn download_model(&self, model_id: &str, destination: &Path) -> Result<()> {
        fs::create_dir_all(destination).await?;

        println!("Fetching model info for {}...", model_id);
        let info = self.get_model_info(model_id).await?;

        if let Some(size) = info.size_bytes {
            println!("Model size: {:.1} GB\n", size as f64 / 1_000_000_000.0);
        }

        let gguf_files: Vec<String> = info.files.iter()
            .filter(|f| f.ends_with(".gguf"))
            .cloned()
            .collect();

        if gguf_files.is_empty() {
            return Err(anyhow::anyhow!(
                "No GGUF files found for {}.\n\
                 Capi currently supports GGUF models only.\n\
                 Look for models with '-gguf' or '-GGUF' in the name.",
                model_id
            ));
        }

        println!("Downloading {} GGUF file(s)...\n", gguf_files.len());

        for (idx, file_name) in gguf_files.iter().enumerate() {
            let url = format!(
                "https://huggingface.co/{}/resolve/main/{}",
                model_id, file_name
            );

            println!("[{}/{}] {}...", idx + 1, gguf_files.len(), file_name);

            match self.download_file_with_progress(&url, &destination.join(file_name), |current, total| {
                if total > 0 {
                    let pct = (current as f64 / total as f64 * 100.0) as u32;
                    let mb_current = current as f64 / 1_000_000.0;
                    let mb_total = total as f64 / 1_000_000.0;
                    print!("\r  {:.1}/{:.1} MB ({}%)  ", mb_current, mb_total, pct);
                    use std::io::Write;
                    std::io::stdout().flush().ok();
                }
            }).await {
                Ok(_) => println!("\r  ✓ {} downloaded", file_name),
                Err(e) => {
                    eprintln!("\r  ✗ Failed: {}", e);
                }
            }
        }

        println!("\n✓ Download complete");
        Ok(())
    }

    // download_file removed as unused

    pub async fn download_file_with_progress<F>(
        &self,
        url: &str,
        destination: &Path,
        mut progress_callback: F,
    ) -> Result<()>
    where
        F: FnMut(u64, u64),
    {
        let response = self.client.get(url).send().await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("HTTP error: {}", response.status()));
        }

        let total_size = response.content_length().unwrap_or(0);
        let mut file = File::create(destination).await?;
        let mut downloaded: u64 = 0;

        let mut stream = response.bytes_stream();
        use futures::StreamExt;

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            file.write_all(&chunk).await?;
            downloaded += chunk.len() as u64;
            progress_callback(downloaded, total_size);
        }

        Ok(())
    }
}

fn is_supported_model(model: &HfApiModel) -> bool {
    let tags = model.tags.as_ref().map(|t| t.as_slice()).unwrap_or(&[]);

    let has_incompatible = tags.iter().any(|tag| {
        let tag_lower = tag.to_lowercase();
        tag_lower.contains("gptq") ||
        tag_lower.contains("awq") ||
        tag_lower.contains("bnb") ||
        tag_lower.contains("exl2")
    });

    !has_incompatible
}
