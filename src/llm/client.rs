use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct ChatRequest<'a> {
    model: &'a str,
    messages: &'a [Message<'a>],
    temperature: f64,
    max_tokens: u32,
}

#[derive(Serialize)]
struct Message<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: ChoiceMessage,
}

#[derive(Deserialize)]
struct ChoiceMessage {
    content: Option<String>,
}

#[derive(Clone)]
pub struct LlmClient {
    client: Client,
    api_key: String,
    base_url: String,
}

impl LlmClient {
    pub fn new(api_key: &str) -> Self {
        Self {
            client: Client::new(),
            api_key: api_key.to_string(),
            base_url: "https://opencode.ai/zen/go/v1".to_string(),
        }
    }

    pub async fn chat(
        &self,
        system_prompt: &str,
        user_prompt: &str,
        temperature: f64,
        max_tokens: u32,
    ) -> Result<String> {
        let messages = vec![
            Message { role: "system", content: system_prompt },
            Message { role: "user", content: user_prompt },
        ];

        let request = ChatRequest {
            model: "deepseek-v4-pro",
            messages: &messages,
            temperature,
            max_tokens,
        };

        let response = self
            .client
            .post(format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&request)
            .send()
            .await
            .context("LLM request failed")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("LLM error {}: {}", status, body);
        }

        let body: ChatResponse = response.json().await.context("Failed to parse LLM response")?;

        body.choices
            .first()
            .and_then(|c| c.message.content.clone())
            .context("LLM returned empty content")
    }

    pub async fn generate_chapter(
        &self,
        system_prompt: &str,
        user_prompt: &str,
    ) -> Result<String> {
        self.chat(system_prompt, user_prompt, 0.7, 8192).await
    }

    pub async fn generate_summary(&self, chapter_content: &str) -> Result<String> {
        let system = "你是一个故事摘要助手。根据章节内容生成100字以内的剧情摘要，只描述发生了什么事，不做评价。";
        let user = format!("请概括以下章节的关键剧情：\n\n{}", chapter_content);
        self.chat(system, &user, 0.3, 512).await
    }
}
