// https://doc.rust-lang.org/reference/items/modules.html#module-source-filenames
// Preferred way is to name modules with their subfolder name now (no longer mod.rs)
pub mod opcodes;
pub mod packet_capture;
mod packet_process;
pub mod utils;
#[cfg(target_os = "windows")]
pub mod read_packets_windows;
#[cfg(target_os = "windows")]
pub use read_packets_windows as read_packets;
#[cfg(target_os = "linux")]
pub mod read_packets_linux;
#[cfg(target_os = "linux")]
pub use read_packets_linux as read_packets;
