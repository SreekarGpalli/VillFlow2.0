//! Audio capture via WASAPI (through `cpal`) with resampling to 16 kHz PCM s16le.

use bytes::Bytes;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use serde::Serialize;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::mpsc;

// ---------------------------------------------------------------------------
// Error
// ---------------------------------------------------------------------------

/// Errors from audio device enumeration or capture.
#[derive(Debug, Error)]
pub enum AudioError {
    #[error("No audio input devices found")]
    NoInputDevices,

    #[error("Device not found: {0}")]
    DeviceNotFound(String),

    #[error("Failed to get default input config: {0}")]
    DefaultConfig(String),

    #[error("Failed to build input stream: {0}")]
    BuildStream(String),

    #[error("Stream play error: {0}")]
    PlayStream(String),

    #[error("cpal host error: {0}")]
    Host(String),

    #[error("Resampler error: {0}")]
    Resampler(String),
}

// ---------------------------------------------------------------------------
// Device info
// ---------------------------------------------------------------------------

/// Describes an available audio input device.
#[derive(Debug, Clone, Serialize)]
pub struct AudioDevice {
    /// Human-readable device name.
    pub name: String,
    /// Whether this is the system default input device.
    pub is_default: bool,
}

/// List all available audio input devices on the system.
pub fn enumerate_devices() -> Result<Vec<AudioDevice>, AudioError> {
    let host = cpal::default_host();
    let default_device_name = host
        .default_input_device()
        .and_then(|d| d.name().ok());

    let mut devices = Vec::new();
    let input_devices = host
        .input_devices()
        .map_err(|e| AudioError::Host(e.to_string()))?;

    for device in input_devices {
        if let Ok(name) = device.name() {
            let is_default = default_device_name
                .as_ref()
                .map(|d| d == &name)
                .unwrap_or(false);
            devices.push(AudioDevice { name, is_default });
        }
    }

    if devices.is_empty() {
        return Err(AudioError::NoInputDevices);
    }
    Ok(devices)
}

// ---------------------------------------------------------------------------
// Capture handle
// ---------------------------------------------------------------------------

/// A handle to a running audio capture session.
///
/// Call [`stop()`](AudioCaptureHandle::stop) to end the capture.
#[derive(Debug)]
pub struct AudioCaptureHandle {
    /// Signal the capture thread to stop.
    stop_flag: Arc<AtomicBool>,
    /// Join handle for the capture thread.
    thread_handle: Option<std::thread::JoinHandle<()>>,
}

impl AudioCaptureHandle {
    /// Stop the capture stream and wait for the background thread to exit.
    pub fn stop(&mut self) {
        self.stop_flag.store(true, Ordering::SeqCst);
        if let Some(handle) = self.thread_handle.take() {
            let _ = handle.join();
        }
    }
}

impl Drop for AudioCaptureHandle {
    fn drop(&mut self) {
        self.stop();
    }
}

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Target sample rate for the Speechmatics stream.
pub const TARGET_SAMPLE_RATE: u32 = 16_000;

/// Number of frames per resampling chunk. Rubato works in fixed-size blocks.
const RESAMPLE_CHUNK_FRAMES: usize = 1024;

// ---------------------------------------------------------------------------
// Capture entry point
// ---------------------------------------------------------------------------

/// Start capturing audio from the given device (or default).
///
/// Audio data is sent as **PCM s16le, mono, 16 kHz** [`Bytes`] chunks through
/// the returned channel.
pub fn start_capture(
    device_name: Option<&str>,
    tx: mpsc::Sender<Bytes>,
) -> Result<AudioCaptureHandle, AudioError> {
    let host = cpal::default_host();

    // Resolve device
    let device = match device_name {
        Some(name) if name != "default" => {
            let input_devices = host
                .input_devices()
                .map_err(|e| AudioError::Host(e.to_string()))?;
            let mut found = None;
            for d in input_devices {
                if d.name().ok().as_deref() == Some(name) {
                    found = Some(d);
                    break;
                }
            }
            found.ok_or_else(|| AudioError::DeviceNotFound(name.to_owned()))?
        }
        _ => host
            .default_input_device()
            .ok_or(AudioError::NoInputDevices)?,
    };

    let config = device
        .default_input_config()
        .map_err(|e| AudioError::DefaultConfig(e.to_string()))?;

    let source_rate = config.sample_rate().0;
    let source_channels = config.channels() as usize;

    tracing::info!(
        "Audio capture: device={:?}, rate={source_rate}, channels={source_channels}",
        device.name().unwrap_or_default()
    );

    let stop_flag = Arc::new(AtomicBool::new(false));
    let stop_flag_clone = Arc::clone(&stop_flag);

    let thread_handle = std::thread::spawn(move || {
        if let Err(e) = capture_thread(
            device,
            config.into(),
            source_rate,
            source_channels,
            tx,
            stop_flag_clone,
        ) {
            tracing::error!("Audio capture thread error: {e}");
        }
    });

    Ok(AudioCaptureHandle {
        stop_flag,
        thread_handle: Some(thread_handle),
    })
}

// ---------------------------------------------------------------------------
// Internal capture thread
// ---------------------------------------------------------------------------

fn capture_thread(
    device: cpal::Device,
    config: cpal::StreamConfig,
    source_rate: u32,
    source_channels: usize,
    tx: mpsc::Sender<Bytes>,
    stop_flag: Arc<AtomicBool>,
) -> Result<(), AudioError> {
    use rubato::{FftFixedIn, Resampler};

    let needs_resample = source_rate != TARGET_SAMPLE_RATE;

    // Build resampler if needed
    let mut resampler: Option<FftFixedIn<f32>> = if needs_resample {
        Some(
            FftFixedIn::<f32>::new(
                source_rate as usize,
                TARGET_SAMPLE_RATE as usize,
                RESAMPLE_CHUNK_FRAMES,
                1, // sub-chunks
                1, // mono
            )
            .map_err(|e| AudioError::Resampler(e.to_string()))?,
        )
    } else {
        None
    };

    // Accumulator for resampler input (mono f32)
    let mut mono_buf: Vec<f32> = Vec::with_capacity(RESAMPLE_CHUNK_FRAMES * 2);
    // Pre-allocated chunk buffer for resampler input to avoid allocations per frame
    let mut chunk_in = vec![0.0f32; RESAMPLE_CHUNK_FRAMES];

    let tx_data = tx.clone();
    let mut callback_count = 0u64;

    let err_fn = |err: cpal::StreamError| {
        tracing::error!("cpal stream error: {err}");
    };

    let stream = device
        .build_input_stream(
            &config,
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                callback_count = callback_count.wrapping_add(1);

                // Down-mix to mono
                for chunk in data.chunks(source_channels) {
                    let sample: f32 =
                        chunk.iter().copied().sum::<f32>() / source_channels as f32;
                    mono_buf.push(sample);
                }

                // Add a maximum capacity check
                if mono_buf.len() > RESAMPLE_CHUNK_FRAMES * 16 {
                    let to_drain = mono_buf.len() - RESAMPLE_CHUNK_FRAMES * 16;
                    mono_buf.drain(..to_drain);
                }

                // If we don't need resampling, convert and send directly
                if !needs_resample {
                    let pcm = f32_to_pcm_s16le(&mono_buf);
                    mono_buf.clear();
                    mono_buf.shrink_to(RESAMPLE_CHUNK_FRAMES * 2);
                    if let Err(e) = tx_data.try_send(Bytes::from(pcm)) {
                        tracing::warn!("Audio capture channel full, dropping chunk: {e}");
                    }
                    return;
                }

                // Feed resampler in fixed-size chunks
                if let Some(ref mut rs) = resampler {
                    while mono_buf.len() >= RESAMPLE_CHUNK_FRAMES {
                        // Copy data without re-allocating
                        chunk_in.copy_from_slice(&mono_buf[..RESAMPLE_CHUNK_FRAMES]);
                        mono_buf.drain(..RESAMPLE_CHUNK_FRAMES);

                        match rs.process(&[&chunk_in], None) {
                            Ok(output) => {
                                if let Some(resampled) = output.first() {
                                    let pcm = f32_to_pcm_s16le(resampled);
                                    if let Err(e) = tx_data.try_send(Bytes::from(pcm)) {
                                        tracing::warn!("Audio capture channel full, dropping chunk: {e}");
                                    }
                                }
                            }
                            Err(e) => {
                                tracing::warn!("Resampler error: {e}");
                            }
                        }
                    }
                    if callback_count % 128 == 0 {
                        mono_buf.shrink_to(RESAMPLE_CHUNK_FRAMES * 4);
                    }
                }
            },
            err_fn,
            None,
        )
        .map_err(|e| AudioError::BuildStream(e.to_string()))?;

    stream
        .play()
        .map_err(|e| AudioError::PlayStream(e.to_string()))?;

    // Wait until the stop flag is set
    while !stop_flag.load(Ordering::SeqCst) {
        std::thread::sleep(std::time::Duration::from_millis(50));
    }

    // `stream` is dropped here, stopping capture.
    tracing::info!("Audio capture stopped");
    Ok(())
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Convert f32 PCM samples (range -1.0..1.0) to little-endian i16 bytes.
fn f32_to_pcm_s16le(samples: &[f32]) -> Vec<u8> {
    let mut out = Vec::with_capacity(samples.len() * 2);
    for &s in samples {
        let clamped = s.clamp(-1.0, 1.0);
        let i = (clamped * i16::MAX as f32) as i16;
        out.extend_from_slice(&i.to_le_bytes());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_f32_to_pcm_s16le_edge_cases() {
        let input = vec![0.0, 1.0, -1.0, 2.0, -2.0, 0.5, -0.5];
        let output = f32_to_pcm_s16le(&input);
        
        // 0.0 -> 0, 0 in bytes
        assert_eq!(output[0..2], 0i16.to_le_bytes());
        // 1.0 -> i16::MAX
        assert_eq!(output[2..4], i16::MAX.to_le_bytes());
        // -1.0 -> -32767
        assert_eq!(output[4..6], (-32767i16).to_le_bytes());
        // 2.0 -> clamped to 1.0 -> i16::MAX
        assert_eq!(output[6..8], i16::MAX.to_le_bytes());
        // -2.0 -> clamped to -1.0 -> -32767
        assert_eq!(output[8..10], (-32767i16).to_le_bytes());
    }
}
