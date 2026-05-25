//! LLM integration module (Groq API).

pub mod groq;

pub use groq::{cleanup_transcript, LlmError};
