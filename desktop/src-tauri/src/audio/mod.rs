pub mod analyser;
pub mod commands;
mod decode;
mod device;
mod engine;
mod eq;
mod state;
mod tick;
mod timing;
mod types;

// souvlaki (MPRIS/SMTC) недоступен на Android — используем noop
#[cfg(not(target_os = "android"))]
mod media_controls;

pub use analyser::start_fft_thread;
pub use commands::*;
pub use device::start_default_output_monitor;
pub use state::init;
pub use tick::start_tick_emitter;

#[cfg(not(target_os = "android"))]
pub use media_controls::start_media_controls;

/// На Android системных медиа-контролов нет — noop
#[cfg(target_os = "android")]
pub fn start_media_controls(_app: &tauri::AppHandle) {}
