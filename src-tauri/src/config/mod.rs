//! Configuration module — application settings persistence.

mod settings;

pub use settings::{
    config_dir, load_settings, log_dir, save_settings, AppSettings, ConfigError,
    InjectionMethod, SpeechmaticsRegion, OperatingPoint, register_startup,
};
