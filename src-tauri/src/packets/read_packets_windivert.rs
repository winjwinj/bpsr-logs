use crate::packets;

use crate::packets::opcodes::Pkt;
use crate::packets::packet_process::process_packet;
use crate::packets::utils::{BinaryReader, Server, TCPReassembler};
use etherparse::NetSlice::Ipv4;
use etherparse::SlicedPacket;
use etherparse::TransportSlice::Tcp;
use log::{debug, error, info, warn};
use tokio::sync::watch;
use windivert::WinDivert;
use windivert::prelude::WinDivertFlags;
use crate::packets::packet_capture::packet_handler;

#[allow(clippy::too_many_lines)]
pub async fn read_packets(
    packet_sender: &tokio::sync::mpsc::Sender<(packets::opcodes::Pkt, Vec<u8>)>,
    restart_receiver: &mut watch::Receiver<bool>,
) {
    let windivert = match WinDivert::network(
        "!loopback && ip && tcp", // todo: idk why but filtering by port just crashes the program, investigate?
        0,
        WinDivertFlags::new().set_sniff(),
    ) {
        Ok(windivert_handle) => {
            info!("WinDivert handle opened!");
            Some(windivert_handle)
        }
        Err(e) => {
            error!("Failed to initialize WinDivert: {}", e);
            return;
        }
    }
        .expect("Failed to initialize WinDivert"); // if windivert doesn't work just exit early - todo: maybe we want to log this with a match so its clearer?
    let mut windivert_buffer = vec![0u8; 10 * 1024 * 1024];
    let mut known_server: Option<Server> = None;
    let mut tcp_reassembler: TCPReassembler = TCPReassembler::new();
    while let Ok(packet) = windivert.recv(Some(&mut windivert_buffer)) {
        // info!("{}", line!());
        let Ok(network_slices) = SlicedPacket::from_ip(packet.data.as_ref()) else {
            continue; // if it's not ip, go next packet
        };
        packet_handler(&mut known_server, &mut tcp_reassembler, packet_sender, network_slices).await;

        if *restart_receiver.borrow() {
            return;
        }
    } // todo: if it errors, it breaks out of the loop but will it ever error?
    // info!("{}", line!());
}