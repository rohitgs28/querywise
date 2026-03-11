use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub enum AiProvider {
    Anthropic { api_key: String },
    OpenAi { api_key: String },
    Ollama { url: String, model: String },
}

#[derive(Debug, Serialize)]
struct AnthropicRequest {
    model: String,
    max_tokens: u32,
    temperature: f32,
    system: String,
    messages: Vec<Message>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct AnthropicResponse {
    content: Vec<ContentBlock>,
}

#[derive(Debug, Deserialize)]
struct ContentBlock {
    text: Option<String>,
}

impl AiProvider {
    pub fn from_config(provider: &str, config: &crate::config::AppConfig) -> Result<Self> {
        match provider {
            "anthropic" => {
                let key = config
                    .anthropic_api_key
                    .clone()
                    .or_else(|| std::env::var("ANTHROPIC_API_KEY").ok())
                    .ok_or_else(|| {
                        anyhow::anyhow!(
                            "ANTHROPIC_API_KEY not set. Set it via environment variable \
                             or in ~/.config/querywise/config.toml"
                        )
                    })?;
                Ok(AiProvider::Anthropic { api_key: key })
            }
            "openai" => {
                let key = config
                    .openai_api_key
                    .clone()
                    .or_else(|| std::env::var("OPENAI_API_KEY").ok())
                    .ok_or_else(|| anyhow::anyhow!("OPENAI_API_KEY not set"))?;
                Ok(AiProvider::OpenAi { api_key: key })
            }
            "ollama" => {
                let url = config
                    .ollama_url
                    .clone()
                    .unwrap_or_else(|| "http://localhost:11434".to_string());
                let model = config
                    .ollama_model
                    .clone()
                    .or_else(|| std::env::var("OLLAMA_MODEL").ok())
                    .unwrap_or_else(|| "llama3".to_string());
                Ok(AiProvider::Ollama { url, model })
            }
            _ => Err(anyhow::anyhow!("Unknown AI provider: {}", provider)),
        }
    }

    pub async fn generate(&self, system: &str, user_message: &str) -> Result<String> {
        match self {
            AiProvider::Anthropic { api_key } => {
                let client = reqwest::Client::new();
                let body = AnthropicRequest {
                    model: "claude-sonnet-4-20250514".to_string(),
                    max_tokens: 2048,
                    temperature: 0.0,
                    system: system.to_string(),
                    messages: vec![Message {
                        role: "user".to_string(),
                        content: user_message.to_string(),
                    }],
                };

                let resp = client
                    .post("https://api.anthropic.com/v1/messages")
                    .header("x-api-key", api_key)
                    .header("anthropic-version", "2023-06-01")
                    .header("content-type", "application/json")
                    .json(&body)
                    .send()
                    .await?;

                let status = resp.status();
                let text = resp.text().await?;

                if !status.is_success() {
                    return Err(anyhow::anyhow!("AI API error ({}): {}", status, text));
                }

                let parsed: AnthropicResponse = serde_json::from_str(&text)?;
                Ok(parsed
                    .content
                    .first()
                    .and_then(|c| c.text.clone())
                    .unwrap_or_default()
                    .trim()
                    .to_string())
            }

            AiProvider::OpenAi { api_key } => {
                let client = reqwest::Client::new();
                let body = serde_json::json!({
                    "model": "gpt-4o",
                    "temperature": 0.0,
                    "max_tokens": 2048,
                    "messages": [
                        {"role": "system", "content": system},
                        {"role": "user", "content": user_message}
                    ]
                });

                let resp = client
                    .post("https://api.openai.com/v1/chat/completions")
                    .header("Authorization", format!("Bearer {}", api_key))
                    .json(&body)
                    .send()
                    .await?;

                let data: serde_json::Value = resp.json().await?;
                Ok(data["choices"][0]["message"]["content"]
                    .as_str()
                    .unwrap_or("")
                    .trim()
                    .to_string())
            }

            AiProvider::Ollama { url, model } => {
                let client = reqwest::Client::new();
                let body = serde_json::json!({
                    "model": model,
                    "stream": false,
                    "system": system,
                    "prompt": user_message
                });

                let resp = client
                    .post(format!("{}/api/generate", url))
                    .json(&body)
                    .send()
                    .await?;

                let data: serde_json::Value = resp.json().await?;
                Ok(data["response"].as_str().unwrap_or("").trim().to_string())
            }
        }
    }
}
