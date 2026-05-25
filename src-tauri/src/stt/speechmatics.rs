//! Speechmatics real-time STT over WebSocket.

use bytes::Bytes;
use futures_util::stream::StreamExt;
use futures_util::sink::SinkExt;
use serde_json::json;
use thiserror::Error;
use tokio::sync::{mpsc, oneshot};
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::Message;
use std::time::Duration;
use crate::config::{SpeechmaticsRegion, OperatingPoint};

const RETRY_BASE_DELAY: Duration = Duration::from_secs(2);
const RETRY_MAX_DELAY: Duration = Duration::from_secs(8);
const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
const READ_TIMEOUT: Duration = Duration::from_secs(30);

type WebSocket = tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;
type WsSink = futures_util::stream::SplitSink<WebSocket, Message>;
type WsStream = futures_util::stream::SplitStream<WebSocket>;

// ---------------------------------------------------------------------------
// Error
// ---------------------------------------------------------------------------

/// Errors from the Speechmatics STT session.
#[derive(Debug, Error)]
pub enum SttError {
    #[error("WebSocket error: {0}")]
    WebSocket(String),

    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Auth rejected by server: {0}")]
    AuthRejected(String),

    #[error("Unexpected server message: {0}")]
    UnexpectedMessage(String),

    #[error("Session cancelled")]
    Cancelled,
}

// ---------------------------------------------------------------------------
// Config
// ---------------------------------------------------------------------------

/// Configuration needed to start a Speechmatics session.
#[derive(Debug, Clone)]
pub struct SpeechmaticsConfig {
    /// Speechmatics API key.
    pub api_key: String,
    /// Region code.
    pub region: SpeechmaticsRegion,
    /// Operating point.
    pub operating_point: OperatingPoint,
    /// Language code (e.g. `"en"`, `"de"`, `"fr"`).
    pub language: String,
    /// Audio sample rate in Hz (should be 16000).
    pub sample_rate: u32,
}

// ---------------------------------------------------------------------------
// Transcript receiver
// ---------------------------------------------------------------------------

/// A handle that resolves to the final accumulated transcript.
#[derive(Debug)]
pub struct TranscriptReceiver {
    rx: oneshot::Receiver<Result<String, String>>,
    abort_handle: Option<tokio::task::AbortHandle>,
}

impl TranscriptReceiver {
    /// Await the final transcript text.
    pub async fn wait(mut self) -> Result<String, String> {
        let (tx_dummy, rx_dummy) = oneshot::channel();
        let rx = std::mem::replace(&mut self.rx, rx_dummy);
        drop(tx_dummy);

        let res = match rx.await {
            Ok(Ok(text)) => Ok(text),
            Ok(Err(e)) => Err(e),
            Err(_) => Err("Speechmatics session was cancelled".to_string()),
        };

        // Clear abort handle on successful completion so drop doesn't call abort
        self.abort_handle = None;
        res
    }
}

impl Drop for TranscriptReceiver {
    fn drop(&mut self) {
        if let Some(ref handle) = self.abort_handle {
            handle.abort();
        }
    }
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Start a real-time Speechmatics session.
///
/// Audio chunks arrive via `audio_rx`. When the sender drops (or sends all
/// data), the session sends `EndOfStream` and waits for the final transcript.
///
/// Returns a [`TranscriptReceiver`] that resolves to the accumulated transcript
/// once the server sends `EndOfTranscript`.
pub fn start_session(
    config: SpeechmaticsConfig,
    audio_rx: mpsc::Receiver<Bytes>,
) -> Result<TranscriptReceiver, SttError> {
    let (tx_result, rx_result) = oneshot::channel::<Result<String, String>>();

    let join_handle = tokio::spawn(async move {
        session_task(config, audio_rx, tx_result).await;
    });

    Ok(TranscriptReceiver {
        rx: rx_result,
        abort_handle: Some(join_handle.abort_handle()),
    })
}

// ---------------------------------------------------------------------------
// Session task
// ---------------------------------------------------------------------------

async fn session_task(
    config: SpeechmaticsConfig,
    audio_rx: mpsc::Receiver<Bytes>,
    tx_result: oneshot::Sender<Result<String, String>>,
) {
    match session_task_impl(config, audio_rx).await {
        Ok(text) => {
            let _ = tx_result.send(Ok(text));
        }
        Err(e) => {
            let err_msg = e.to_string();
            let _ = tx_result.send(Err(err_msg));
        }
    }
}

fn validate_region(region: SpeechmaticsRegion) -> bool {
    matches!(region, SpeechmaticsRegion::Eu | SpeechmaticsRegion::Usa | SpeechmaticsRegion::Au)
}

fn build_ws_request(config: &SpeechmaticsConfig) -> Result<tokio_tungstenite::tungstenite::handshake::client::Request, SttError> {
    let region_str = match config.region {
        SpeechmaticsRegion::Eu => "eu",
        SpeechmaticsRegion::Usa => "usa",
        SpeechmaticsRegion::Au => "au",
    };
    let url = format!("wss://{}.rt.speechmatics.com/v2", region_str);
    let mut request = url.into_client_request().map_err(|e| SttError::ConnectionFailed(e.to_string()))?;
    let auth_header_val = format!("Bearer {}", config.api_key)
        .parse::<http::header::HeaderValue>()
        .map_err(|e| SttError::ConnectionFailed(e.to_string()))?;
    request.headers_mut().insert("Authorization", auth_header_val);
    Ok(request)
}

async fn connect_with_retry(request: tokio_tungstenite::tungstenite::handshake::client::Request) -> Result<WebSocket, SttError> {
    let mut delay = RETRY_BASE_DELAY;
    let start_time = std::time::Instant::now();
    loop {
        let mut req_clone = http::Request::builder()
            .method(request.method().clone())
            .uri(request.uri().clone())
            .version(request.version());
        if let Some(headers) = req_clone.headers_mut() {
            *headers = request.headers().clone();
        }
        let req_clone = req_clone.body(()).unwrap();

        match tokio_tungstenite::connect_async(req_clone).await {
            Ok((stream, _res)) => return Ok(stream),
            Err(e) => {
                if start_time.elapsed() >= CONNECT_TIMEOUT {
                    return Err(SttError::ConnectionFailed(format!("Connection failed after multiple retries: {e}")));
                }
                tokio::time::sleep(delay).await;
                delay = std::cmp::min(delay * 2, RETRY_MAX_DELAY);
            }
        }
    }
}

async fn read_start_recognition_response(ws_rx: &mut WsStream) -> Result<(), SttError> {
    loop {
        let msg_res = tokio::time::timeout(READ_TIMEOUT, ws_rx.next()).await;
        let msg = match msg_res {
            Ok(Some(Ok(m))) => m,
            Ok(Some(Err(e))) => return Err(SttError::WebSocket(e.to_string())),
            Ok(None) => return Err(SttError::WebSocket("Connection closed before recognition started".to_string())),
            Err(_) => return Err(SttError::WebSocket("Read timeout".to_string())),
        };

        if let Message::Text(txt) = msg {
            let v: serde_json::Value = serde_json::from_str(&txt)?;
            let msg_type = v["message"].as_str().unwrap_or("");
            if msg_type == "RecognitionStarted" {
                tracing::info!("Speechmatics: RecognitionStarted");
                return Ok(());
            } else if msg_type == "Error" {
                return Err(SttError::AuthRejected(txt.to_string()));
            } else {
                tracing::info!("Speechmatics message before start: {msg_type}");
            }
        }
    }
}

async fn stream_audio(mut ws_tx: WsSink, mut audio_rx: mpsc::Receiver<Bytes>) -> u64 {
    let mut seq_no = 0u64;
    while let Some(chunk) = audio_rx.recv().await {
        seq_no += 1;
        if let Err(e) = ws_tx
            .send(Message::Binary(chunk.to_vec().into()))
            .await
        {
            tracing::warn!("Failed to send audio: {e}");
            seq_no -= 1;
            break;
        }
    }

    let eos = json!({
        "message": "EndOfStream",
        "last_seq_no": seq_no
    });
    if let Err(e) = ws_tx
        .send(Message::Text(eos.to_string().into()))
        .await
    {
        tracing::warn!("Failed to send EndOfStream: {e}");
    }
    tracing::debug!("Sent EndOfStream with last_seq_no={seq_no}");

    seq_no
}

async fn session_task_impl(
    mut config: SpeechmaticsConfig,
    audio_rx: mpsc::Receiver<Bytes>,
) -> Result<String, SttError> {
    if !validate_region(config.region) {
        crate::credentials::zeroize_string(&mut config.api_key);
        return Err(SttError::ConnectionFailed(format!("Invalid Speechmatics region: {:?}", config.region)));
    }

    let request_res = build_ws_request(&config);
    crate::credentials::zeroize_string(&mut config.api_key);
    let request = request_res?;

    let ws_stream = connect_with_retry(request).await?;
    tracing::info!("Connected to Speechmatics WebSocket");

    let (mut ws_tx, mut ws_rx) = ws_stream.split();

    let operating_point_str = match config.operating_point {
        OperatingPoint::Standard => "standard",
        OperatingPoint::Enhanced => "enhanced",
    };
    let start_msg = json!({
        "message": "StartRecognition",
        "audio_format": {
            "type": "raw",
            "encoding": "pcm_s16le",
            "sample_rate": config.sample_rate
        },
        "transcription_config": {
            "language": config.language,
            "enable_partials": true,
            "operating_point": operating_point_str
        }
    });

    ws_tx
        .send(Message::Text(start_msg.to_string().into()))
        .await
        .map_err(|e| SttError::WebSocket(e.to_string()))?;

    tracing::debug!("Sent StartRecognition");

    read_start_recognition_response(&mut ws_rx).await?;

    let send_handle = tokio::spawn(async move {
        stream_audio(ws_tx, audio_rx).await
    });

    // ── Read transcript messages ─────────────────────────────────────────
    let mut final_transcript = String::new();
    let mut session_ended = false;
    let mut err = None;

    loop {
        let msg_res = tokio::time::timeout(READ_TIMEOUT, ws_rx.next()).await;
        let msg = match msg_res {
            Ok(Some(msg_val)) => msg_val,
            Ok(None) => {
                break;
            }
            Err(_) => {
                err = Some(SttError::WebSocket("Read timeout".to_string()));
                break;
            }
        };

        match msg {
            Ok(Message::Text(txt)) => {
                let v: serde_json::Value = serde_json::from_str(&txt)?;
                let msg_type = v["message"].as_str().unwrap_or("");

                match msg_type {
                    "AddTranscript" => {
                        if let Some(text) = extract_transcript_text(&v) {
                            final_transcript.push_str(&text);
                            tracing::debug!("Final chunk: {text}");
                        }
                    }
                    "AddPartialTranscript" => {
                        tracing::trace!("Partial transcript received");
                    }
                    "EndOfTranscript" => {
                        tracing::info!("EndOfTranscript received");
                        session_ended = true;
                        break;
                    }
                    "Error" => {
                        tracing::error!("Speechmatics error: {txt}");
                        err = Some(SttError::WebSocket(format!("Speechmatics API error: {txt}")));
                        break;
                    }
                    other => {
                        tracing::trace!("Speechmatics message: {other}");
                    }
                }
            }
            Ok(Message::Close(reason)) => {
                tracing::info!("WebSocket closed: {:?}", reason);
                err = Some(SttError::WebSocket(format!("WebSocket closed: {:?}", reason)));
                break;
            }
            Err(e) => {
                tracing::error!("WebSocket read error: {e}");
                err = Some(SttError::WebSocket(e.to_string()));
                break;
            }
            _ => {}
        }
    }

    if !session_ended {
        send_handle.abort();
        tracing::warn!("Session ended without EndOfTranscript");
        let error_to_return = err.unwrap_or_else(|| {
            SttError::WebSocket("Connection closed by server before receiving EndOfTranscript".to_string())
        });
        return Err(error_to_return);
    } else {
        let _ = send_handle.await;
    }

    let trimmed = final_transcript.trim().to_owned();
    tracing::info!("Final transcript length: {} chars", trimmed.len());
    Ok(trimmed)
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Extract the concatenated text from a Speechmatics `AddTranscript` message.
fn extract_transcript_text(v: &serde_json::Value) -> Option<String> {
    let results = v["results"].as_array()?;
    let mut text = String::new();
    for r in results {
        if let Some(content) = r["alternatives"][0]["content"].as_str() {
            // Speechmatics uses "word" type; insert spaces between words
            if !text.is_empty() && r["type"].as_str() == Some("word") {
                text.push(' ');
            }
            text.push_str(content);
        }
    }
    Some(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_transcript_text() {
        let input = serde_json::json!({
            "results": [
                {
                    "alternatives": [{ "content": "hello" }],
                    "type": "word"
                },
                {
                    "alternatives": [{ "content": "world" }],
                    "type": "word"
                },
                {
                    "alternatives": [{ "content": "." }],
                    "type": "punctuation"
                }
            ]
        });
        let result = extract_transcript_text(&input);
        assert_eq!(result, Some("hello world.".to_string()));
        
        let empty_input = serde_json::json!({ "results": [] });
        assert_eq!(extract_transcript_text(&empty_input), Some("".to_string()));
    }
}
