use crate::packets;
use crate::packets::read_packets;
use once_cell::sync::OnceCell;
use tokio::sync::watch;

// Global sender for restart signal
static RESTART_SENDER: OnceCell<watch::Sender<bool>> = OnceCell::new();

pub fn start_capture() -> tokio::sync::mpsc::Receiver<(packets::opcodes::Pkt, Vec<u8>)> {
    let (packet_sender, packet_receiver) =
        tokio::sync::mpsc::channel::<(packets::opcodes::Pkt, Vec<u8>)>(1);
    let (restart_sender, mut restart_receiver) = watch::channel(false);
    RESTART_SENDER.set(restart_sender.clone()).ok();
    tauri::async_runtime::spawn(async move {
        loop {
            read_packets(&packet_sender, &mut restart_receiver).await;
            // Wait for restart signal
            while !*restart_receiver.borrow() {
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            }
            // Reset signal to false before next loop
            let _ = restart_sender.send(false);
        }
        // info!("oopsies {}", line!());
    });
    packet_receiver
}

// Function to send restart signal from another thread/task
pub fn request_restart() {
    if let Some(sender) = RESTART_SENDER.get() {
        let _ = sender.send(true);
    }
}
