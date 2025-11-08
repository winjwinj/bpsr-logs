use etherparse::SlicedPacket;
use crate::packets;
use tokio::sync::watch;
use pcap::Device;
use log::{error, info};
use crate::packets::packet_capture::packet_handler;
use crate::packets::utils::{Server, TCPReassembler};

pub async fn read_packets(
    packet_sender: &tokio::sync::mpsc::Sender<(packets::opcodes::Pkt, Vec<u8>)>,
    restart_receiver: &mut watch::Receiver<bool>,
) {
    let device = match Device::lookup() {
        Ok(Some(dev)) => dev,
        Ok(None) => { 
            error!("No device found.");
            return;
        }
        Err(e) => {
            error!("Failed to lookup device: {}", e);
            return;
        }
    };
    
    info!("using device: {}", device.name);

    let mut cap = match device.open() {
        Ok(c) => c,
        Err(e) => {
            error!("Failed to open device: {}", e);
            return;
        }
    };

    let mut known_server: Option<Server> = None;
    let mut tcp_reassembler: TCPReassembler = TCPReassembler::new();

    while let Ok(packet) = cap.next_packet() {
        let Ok(network_slices) = SlicedPacket::from_ethernet(packet.data) else {
            continue;
        };
        packet_handler(&mut known_server, &mut tcp_reassembler, packet_sender, network_slices).await;
        
        if *restart_receiver.borrow() {
            return;
        }
    }
}