//! Audio capture module.

mod capture;

pub use capture::{
    enumerate_devices, start_capture, AudioCaptureHandle, AudioDevice, AudioError, TARGET_SAMPLE_RATE,
};
