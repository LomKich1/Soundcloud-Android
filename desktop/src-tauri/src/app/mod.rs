pub mod diagnostics;

// Системный трей и popover-окно существуют только на десктопе
#[cfg(not(target_os = "android"))]
pub mod popover;

#[cfg(not(target_os = "android"))]
pub mod tray;
