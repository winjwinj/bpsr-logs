use crate::packets;
use crate::packets::opcodes::Pkt;
use crate::packets::packet_process::process_packet;
use crate::packets::utils::{BinaryReader, Server, TCPReassembler};
use etherparse::NetSlice::Ipv4;
use etherparse::SlicedPacket;
use etherparse::TransportSlice::Tcp;
use log::{debug, error, info, warn};
use once_cell::sync::OnceCell;
use tokio::sync::watch;
use windivert::WinDivert;
use windivert::prelude::WinDivertFlags;

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

#[allow(clippy::too_many_lines)]
async fn read_packets(
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
    let mut known_server: Option<Server> = None; // nothing at start
    let mut tcp_reassembler: TCPReassembler = TCPReassembler::new();
    while let Ok(packet) = windivert.recv(Some(&mut windivert_buffer)) {
        // info!("{}", line!());
        let Ok(network_slices) = SlicedPacket::from_ip(packet.data.as_ref()) else {
            continue; // if it's not ip, go next packet
        };
        // info!("{}", line!());
        let Some(Ipv4(ip_packet)) = network_slices.net else {
            continue;
        };
        // info!("{}", line!());
        let Some(Tcp(tcp_packet)) = network_slices.transport else {
            continue;
        };
        // info!("{}", line!());
        let curr_server = Server::new(
            ip_packet.header().source(),
            tcp_packet.to_header().source_port,
            ip_packet.header().destination(),
            tcp_packet.to_header().destination_port,
        );
        // trace!(
        //     "{} ({}) => {:?}",
        //     curr_server,
        //     tcp_packet.payload().len(),
        //     tcp_packet.payload(),
        // );

        // 1. Try to identify game server via small packets
        if known_server != Some(curr_server) {
            let tcp_payload = tcp_packet.payload();
            let mut tcp_payload_reader = BinaryReader::from(tcp_payload.to_vec());
            if tcp_payload_reader.remaining() >= 10 {
                match tcp_payload_reader.read_bytes(10) {
                    Ok(bytes) => {
                        if bytes[4] == 0 {
                            const FRAG_LENGTH_SIZE: usize = 4;
                            const SIGNATURE: [u8; 6] = [0x00, 0x63, 0x33, 0x53, 0x42, 0x00];
                            let mut i = 0;
                            while tcp_payload_reader.remaining() >= FRAG_LENGTH_SIZE {
                                i += 1;
                                if i > 1000 {
                                    info!(
                                        "Line: {} - Stuck at 1. Try to identify game server via small packets?",
                                        line!()
                                    );
                                }
                                let tcp_frag_payload_len = match tcp_payload_reader.read_u32() {
                                    Ok(len) => len.saturating_sub(FRAG_LENGTH_SIZE as u32) as usize,
                                    Err(e) => {
                                        debug!("Malformed TCP fragment: failed to read_u32: {e}");
                                        break;
                                    }
                                };
                                if tcp_payload_reader.remaining() >= tcp_frag_payload_len {
                                    match tcp_payload_reader.read_bytes(tcp_frag_payload_len) {
                                        Ok(tcp_frag) => {
                                            if tcp_frag.len() >= 5 + SIGNATURE.len()
                                                && tcp_frag[5..5 + SIGNATURE.len()] == SIGNATURE
                                            {
                                                info!(
                                                    "Got Scene Server Address (by change): {curr_server}"
                                                );
                                                known_server = Some(curr_server);
                                                tcp_reassembler.clear_reassembler(
                                                    tcp_packet.sequence_number() as usize
                                                        + tcp_payload_reader.len(),
                                                );
                                                if let Err(err) = packet_sender
                                                    .send((Pkt::ServerChangeInfo, Vec::new()))
                                                    .await
                                                {
                                                    debug!("Failed to send packet: {err}");
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            debug!(
                                                "Malformed TCP fragment: failed to read_bytes: {e}"
                                            );
                                            break;
                                        }
                                    }
                                } else {
                                    break;
                                }
                            }
                        }
                    }
                    Err(e) => {
                        debug!("Malformed TCP payload: failed to read_bytes(10): {e}");
                    }
                }
            }
            // 2. Payload length is 98 = Login packets?
            if tcp_payload.len() == 98 {
                const SIGNATURE_1: [u8; 10] =
                    [0x00, 0x00, 0x00, 0x62, 0x00, 0x03, 0x00, 0x00, 0x00, 0x01];
                const SIGNATURE_2: [u8; 6] = [0x00, 0x00, 0x00, 0x00, 0x0a, 0x4e];
                if tcp_payload.len() >= 20
                    && tcp_payload[0..10] == SIGNATURE_1
                    && tcp_payload[14..20] == SIGNATURE_2
                {
                    info!("Got Scene Server Address by Login Return Packet: {curr_server}");
                    known_server = Some(curr_server);
                    tcp_reassembler.clear_reassembler(
                        tcp_packet.sequence_number() as usize + tcp_payload.len(),
                    );
                    if let Err(err) = packet_sender
                        .send((Pkt::ServerChangeInfo, Vec::new()))
                        .await
                    {
                        debug!("Failed to send packet: {err}");
                    }
                }
            }
            continue;
        }

        if tcp_reassembler.next_seq.is_none() {
            tcp_reassembler.next_seq = Some(tcp_packet.sequence_number() as usize);
        }
        if tcp_reassembler
            .next_seq
            .unwrap()
            .saturating_sub(tcp_packet.sequence_number() as usize)
            == 0
        {
            tcp_reassembler.cache.insert(
                tcp_packet.sequence_number() as usize,
                Vec::from(tcp_packet.payload()),
            );
        }
        let mut i = 0;
        while tcp_reassembler
            .cache
            .contains_key(&tcp_reassembler.next_seq.unwrap())
        {
            i += 1;
            if i % 1000 == 0 {
                warn!(
                    "Potential infinite loop in cache processing: iteration={i}, next_seq={:?}, cache_size={}, _data_len={}",
                    tcp_reassembler.next_seq,
                    tcp_reassembler.cache.len(),
                    tcp_reassembler._data.len()
                );
            }
            let seq = &tcp_reassembler.next_seq.unwrap();
            let cached_tcp_data = tcp_reassembler.cache.get(seq).unwrap();
            if tcp_reassembler._data.is_empty() {
                tcp_reassembler._data = cached_tcp_data.clone();
            } else {
                tcp_reassembler._data.extend_from_slice(cached_tcp_data);
            }
            tcp_reassembler.next_seq = Some(seq.wrapping_add(cached_tcp_data.len()));
            tcp_reassembler.cache.remove(seq);
        }
        i = 0;
        while tcp_reassembler._data.len() > 4 {
            i += 1;
            if i % 1000 == 0 {
                let sample = &tcp_reassembler._data[..tcp_reassembler._data.len().min(32)];
                warn!(
                    "Potential infinite loop in _data processing: iteration={i}, _data_len={}, sample={:?}",
                    tcp_reassembler._data.len(),
                    sample
                );
            }
            let packet_size = match BinaryReader::from(tcp_reassembler._data.clone()).read_u32() {
                Ok(sz) => sz,
                Err(e) => {
                    debug!("Malformed reassembled packet: failed to read_u32: {e}");
                    break;
                }
            };
            if tcp_reassembler._data.len() < packet_size as usize {
                break;
            }
            if tcp_reassembler._data.len() >= packet_size as usize {
                let (left, right) = tcp_reassembler._data.split_at(packet_size as usize);
                let packet = left.to_vec();
                tcp_reassembler._data = right.to_vec();
                debug!(
                    "Processing packet at line {}: size={}",
                    line!(),
                    packet_size
                );
                process_packet(BinaryReader::from(packet), packet_sender.clone()).await;
            }
        }
        if *restart_receiver.borrow() {
            break;
        }
    } // todo: if it errors, it breaks out of the loop but will it ever error?
    // info!("{}", line!());
}

// Function to send restart signal from another thread/task
pub fn request_restart() {
    if let Some(sender) = RESTART_SENDER.get() {
        let _ = sender.send(true);
    }
}
