#[cfg(target_os = "windows")]
mod windivert;

#[cfg(target_os = "windows")]
pub use windivert::{request_restart, start_capture};

#[cfg(not(target_os = "windows"))]
pub fn start_capture() -> tokio::sync::mpsc::Receiver<(crate::packets::opcodes::Pkt, Vec<u8>)> {
    let (_tx, rx) = tokio::sync::mpsc::channel::<(crate::packets::opcodes::Pkt, Vec<u8>)>(1);
    log::info!("Packet capture not available on this platform (Windows only)");
    rx
}

#[cfg(not(target_os = "windows"))]
pub fn request_restart() {
    // No-op on non-Windows platforms
}
