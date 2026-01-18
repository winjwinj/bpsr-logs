use crate::packets;
use crate::packets::opcodes::{FragmentType, Pkt};
use crate::packets::utils::BinaryReader;
use log::debug;

pub fn process_packet(
    mut packets_reader: BinaryReader,
    packet_sender: tokio::sync::mpsc::Sender<(packets::opcodes::Pkt, Vec<u8>)>,
) {
    while packets_reader.remaining() > 0 {
        let packet_size = match packets_reader.peek_u32() {
            Ok(sz) => sz,
            Err(e) => {
                debug!("Malformed packet: failed to peek_u32: {e}");
                continue;
            }
        };
        if packet_size < 6 {
            debug!("Malformed packet: packet_size < 6");
            continue;
        }

        let mut reader = match packets_reader.read_bytes(packet_size as usize) {
            Ok(bytes) => BinaryReader::from(bytes),
            Err(e) => {
                debug!("Malformed packet: failed to read_bytes: {e}");
                continue;
            }
        };
        if reader.read_u32().is_err() {
            debug!("Malformed packet: failed to skip u32");
            continue;
        }
        let packet_type = match reader.read_u16() {
            Ok(pt) => pt,
            Err(e) => {
                debug!("Malformed packet: failed to read_u16: {e}");
                continue;
            }
        };
        let is_zstd_compressed = packet_type & crate::protocol::constants::packet::COMPRESSION_FLAG;
        let msg_type_id = crate::protocol::constants::packet::extract_type(packet_type);

        match packets::opcodes::FragmentType::from(msg_type_id) {
            FragmentType::Notify => {
                let service_uuid = match reader.read_u64() {
                    Ok(su) => su,
                    Err(e) => {
                        debug!("Malformed Notify: failed to read_u64 service_uuid: {e}");
                        continue;
                    }
                };
                let _stub_id = match reader.read_u32() {
                    Ok(sid) => sid,
                    Err(e) => {
                        debug!("Malformed Notify: failed to read_u32 stub_id: {e}");
                        continue;
                    }
                };
                let method_id_raw = match reader.read_u32() {
                    Ok(mid) => mid,
                    Err(e) => {
                        debug!("Malformed Notify: failed to read_u32 method_id: {e}");
                        continue;
                    }
                };

                if service_uuid != crate::protocol::constants::SERVICE_UUID {
                    debug!("Notify: service_uuid mismatch: {service_uuid:x}");
                    continue;
                }

                let msg_payload = reader.read_remaining();
                let mut tcp_fragment_vec = msg_payload.to_vec();
                if is_zstd_compressed != 0 {
                    match zstd::decode_all(tcp_fragment_vec.as_slice()) {
                        Ok(decoded) => tcp_fragment_vec = decoded,
                        Err(e) => {
                            debug!("Notify: zstd decompression failed: {e}");
                            continue;
                        }
                    }
                }

                let method_id = match Pkt::try_from(method_id_raw) {
                    Ok(mid) => mid,
                    Err(_) => {
                        debug!("Notify: Skipping unknown methodId: {method_id_raw}");
                        continue;
                    }
                };

                if packet_sender
                    .try_send((method_id, tcp_fragment_vec))
                    .is_err()
                {
                    // Channel full - silently drop packet
                }
            }
            FragmentType::FrameDown => {
                let _server_sequence_id = match reader.read_u32() {
                    Ok(sid) => sid,
                    Err(e) => {
                        debug!("FrameDown: failed to read_u32 server_sequence_id: {e}");
                        continue;
                    }
                };
                if reader.remaining() == 0 {
                    debug!("FrameDown: reader.remaining() == 0");
                    break;
                }

                let nested_packet = reader.read_remaining();
                if is_zstd_compressed != 0 {
                    match zstd::decode_all(nested_packet) {
                        Ok(tcp_fragment_decompressed) => {
                            packets_reader = BinaryReader::from(tcp_fragment_decompressed);
                        }
                        Err(e) => {
                            debug!("FrameDown: zstd decompression failed: {e}");
                            continue;
                        }
                    }
                } else {
                    packets_reader = BinaryReader::from(Vec::from(nested_packet));
                }
            }
            _ => {
                debug!("Unknown fragment type: {msg_type_id}");
                continue;
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use crate::packets::opcodes::Pkt;
    use crate::packets::packet_process::process_packet;
    use crate::packets::utils::BinaryReader;

    #[test]
    fn test_add() {
        use std::fs;
        let (packet_sender, _) = tokio::sync::mpsc::channel::<(Pkt, Vec<u8>)>(1);
        let filename = "src/packets/test_add_packet.json";
        let v: Vec<u8> = serde_json::from_str(
            &fs::read_to_string(filename).unwrap_or_else(|_| panic!("Failed to open {filename}")),
        )
        .expect("Invalid JSON in test_packet.json");
        process_packet(BinaryReader::from(v), packet_sender);
    }
}
