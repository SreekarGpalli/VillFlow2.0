//! Groq LLM integration for transcript cleanup and command interpretation.

use serde::Deserialize;
use thiserror::Error;
use std::time::Duration;

const API_TIMEOUT: Duration = Duration::from_secs(7);

/// Errors from the Groq LLM API.
#[derive(Debug, Error)]
pub enum LlmError {
    #[error("HTTP request error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Unexpected API response: {0}")]
    UnexpectedResponse(String),

    #[error("API returned error: {0}")]
    ApiError(String),

    #[error("Request timed out")]
    Timeout,
}

// ---------------------------------------------------------------------------
// Constants & Defaults
// ---------------------------------------------------------------------------

/// Strong default prompt for STT transcription cleanup.
pub const DEFAULT_STT_CLEANUP_PROMPT: &str = concat!(
    "You are an expert transcription editor. Fix grammar, punctuation, typos, and sentence structure. ",
    "Remove all filler words (e.g., um, uh, er, like, basically, you know, sort of, kind of). ",
    "Correct misheard words while preserving the user's original intent, meaning, and tone. ",
    "Never summarize the text. Output ONLY the polished transcription. ",
    "Do not include any introductory text, notes, comments, or explanations."
);

/// Strong default prompt for Command Mode interpretation.
pub const DEFAULT_COMMAND_MODE_PROMPT: &str = concat!(
    "You are a precise voice command processor. Interpret the user's spoken command and perform/format it as requested. ",
    "Return ONLY the final output, text, or result of the action. ",
    "Do not explain what you did, and do not include any preamble, conversational text, quotes, or markdown wrappers unless requested. ",
    "If the command is a query, answer it directly and concisely."
);

// ---------------------------------------------------------------------------
// Response types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct ChatCompletion {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: ChoiceMessage,
}

#[derive(Debug, Deserialize)]
struct ChoiceMessage {
    content: String,
}

#[derive(Debug, Deserialize)]
struct GroqModel {
    id: String,
}

#[derive(Debug, Deserialize)]
struct GroqModelsResponse {
    data: Vec<GroqModel>,
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Send a transcript through the Groq chat completions API for cleanup.
///
/// On error or timeout, the caller handles the fallback to raw transcript.
pub async fn cleanup_transcript(
    client: &reqwest::Client,
    transcript: &str,
    system_prompt: &str,
    api_key: &str,
    model: &str,
    temperature: f32,
    max_tokens: u32,
) -> Result<String, LlmError> {
    let prompt_to_use = if system_prompt.trim().is_empty() {
        DEFAULT_STT_CLEANUP_PROMPT
    } else {
        system_prompt
    };

    let body = serde_json::json!({
        "model": model,
        "messages": [
            { "role": "system", "content": prompt_to_use },
            { "role": "user", "content": transcript }
        ],
        "temperature": temperature,
        "max_tokens": max_tokens
    });

    tracing::debug!("Calling Groq API with model={model}");

    let request_fut = client
        .post("https://api.groq.com/openai/v1/chat/completions")
        .header("Authorization", format!("Bearer {api_key}"))
        .header("Content-Type", "application/json")
        .json(&body)
        .send();

    let response = tokio::time::timeout(API_TIMEOUT, request_fut)
        .await
        .map_err(|_| LlmError::Timeout)?
        .map_err(|e| {
            if e.is_timeout() {
                LlmError::Timeout
            } else {
                LlmError::Http(e)
            }
        })?;

    if !response.status().is_success() {
        let status = response.status();
        let body_text = response
            .text()
            .await
            .unwrap_or_else(|_| "<no body>".to_owned());
        return Err(LlmError::ApiError(format!("Status {status}: {body_text}")));
    }

    let completion: ChatCompletion = response.json().await.map_err(|e| {
        LlmError::UnexpectedResponse(e.to_string())
    })?;

    let cleaned = completion
        .choices
        .into_iter()
        .next()
        .map(|c| c.message.content)
        .ok_or_else(|| LlmError::UnexpectedResponse("No choices in Groq response".to_owned()))?;

    let trimmed = cleaned.trim().to_owned();
    tracing::info!(
        "Groq cleanup: {} chars → {} chars",
        transcript.len(),
        trimmed.len()
    );
    Ok(trimmed)
}

/// Fetch list of available models from Groq.
pub async fn fetch_models(client: &reqwest::Client, api_key: &str) -> Result<Vec<String>, LlmError> {
    let response = client
        .get("https://api.groq.com/openai/v1/models")
        .header("Authorization", format!("Bearer {api_key}"))
        .send()
        .await?;

    if !response.status().is_success() {
        let status = response.status();
        let body_text = response
            .text()
            .await
            .unwrap_or_else(|_| "<no body>".to_owned());
        return Err(LlmError::ApiError(format!("Status {status}: {body_text}")));
    }

    let models_resp: GroqModelsResponse = response.json().await?;
    let mut model_ids: Vec<String> = models_resp
        .data
        .into_iter()
        .map(|m| m.id)
        .filter(|id| {
            // Filter out non-chat models (whisper, embedding, vision, guard, moderation)
            !id.contains("whisper")
                && !id.contains("embed")
                && !id.contains("guard")
                && !id.contains("vision-preview")
                && !id.starts_with("distil-")
                && !id.contains("moderation")
        })
        .collect();

    model_ids.sort();
    Ok(model_ids)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chat_completion_deserialization() {
        let json_data = r#"{
            "choices": [
                {
                    "message": {
                        "content": "Hello! I am a polished transcript."
                    }
                }
            ]
        }"#;
        let completion: ChatCompletion = serde_json::from_str(json_data).unwrap();
        assert_eq!(completion.choices[0].message.content, "Hello! I am a polished transcript.");
    }
}
