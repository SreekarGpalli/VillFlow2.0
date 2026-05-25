//! Speech-to-text module (Speechmatics real-time WebSocket).

mod speechmatics;

pub use speechmatics::{
    start_session, SpeechmaticsConfig, SttError, TranscriptReceiver,
};
