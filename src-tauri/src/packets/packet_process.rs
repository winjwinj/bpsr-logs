use crate::packets;
use crate::packets::opcodes::{FragmentType, Pkt};
use crate::packets::utils::BinaryReader;
use log::{debug, info, trace};

pub async fn process_packet(
    mut packets_reader: BinaryReader,
    packet_sender: tokio::sync::mpsc::Sender<(packets::opcodes::Pkt, Vec<u8>)>,
) {
    let mut debug_ctr = 0;
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
        let is_zstd_compressed = packet_type & 0x8000;
        let msg_type_id = packet_type & 0x7fff;

        debug_ctr += 1;
        match packets::opcodes::FragmentType::from(msg_type_id) {
            FragmentType::Notify => {
                let service_uuid = match reader.read_u64() {
                    Ok(su) => su,
                    Err(e) => {
                        debug!("Malformed Notify: failed to read_u64 service_uuid: {e}");
                        continue;
                    }
                };
                let stub_id = match reader.read_u32() {
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

                if service_uuid != 0x0000000063335342 {
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

                if let Err(err) = packet_sender.send((method_id, tcp_fragment_vec)).await {
                    debug!("Failed to send packet: {err}");
                }
            }
            FragmentType::FrameDown => {
                let server_sequence_id = match reader.read_u32() {
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

// pub async fn process_packet(
//     mut tcp_fragments: BinaryReader,
//     packet_sender: tokio::sync::mpsc::Sender<(packets::opcodes::Pkt, Vec<u8>)>,
// ) {
//     println!("during process packet");
//     let mut debug_ctr = 0;
//     const MIN_FRAG_LEN: usize = 8 + 1 + 3; // frag_len + is_zstd + frag_type
//     println!("{}", tcp_fragments.remaining());
//     while tcp_fragments.remaining() >= MIN_FRAG_LEN {
//         let tcp_frag_len = tcp_fragments.peek_u32().unwrap();
//         if tcp_fragments.remaining() < tcp_frag_len as usize {
//             println!("{} < {tcp_frag_len}", tcp_fragments.remaining());
//             return;
//         }
//         let mut tcp_fragment = BinaryReader::from(tcp_fragments.read_bytes(tcp_frag_len as usize).unwrap());
//         let _ = tcp_fragment.read_u32(); // skip tcp_frag_len from before // todo: somehow this crashed before
//
//
//
//         let (is_zstd, frag_type) = {
//             let temp = tcp_fragment.read_u16().unwrap(); // todo: fix all these unwraps properly
//             ((temp & 0x8000) != 0, packets::opcodes::FragmentType::from(temp & 0x7fff)) // get bit 1 and bits 2-16
//         };
//
//         debug_ctr += 1;
//         println!("{frag_type:?}");
//         match frag_type {
//             packets::opcodes::FragmentType::Notify => {
//                 println!("{debug_ctr} Notify {:?}", tcp_fragment.cursor.get_ref());
//                 let service_uuid = tcp_fragment.read_u64().unwrap(); // service_uuid?
//                 let _stub_id = tcp_fragment.read_bytes(4); // bytes 15-18 are ignored
//
//                 if service_uuid == 63_335_342 {
//                     trace!("Skipping FragmentType with service_uuid: {service_uuid}");
//                     return;
//                 }
//
//                 let Ok(method_id) = packets::opcodes::Pkt::try_from(tcp_fragment.read_u32().unwrap()) else {
//                     return;
//                 };
//
//                 let mut tcp_fragment_vec = tcp_fragment.read_remaining().to_vec();
//                 if is_zstd {
//                     if let Ok(decoded) = zstd::decode_all(tcp_fragment_vec.as_slice()) {
//                         tcp_fragment_vec = decoded;
//                     } else {
//                         return; // faulty TCP packet
//                     }
//                 }
//
//                 if let Err(err) = packet_sender.send((method_id, tcp_fragment_vec)).await
//                 {
//                     debug!("Failed to send packet: {err}");
//                 }
//                 break;
//             }
//             packets::opcodes::FragmentType::FrameDown => {
//                 println!("{debug_ctr} FrameDown {:?}", tcp_fragment.cursor.get_ref());
//                 let _ = tcp_fragment.read_bytes(4).unwrap(); // bytes 1-4 are ignored
//                 let tcp_fragment_t = tcp_fragment.read_remaining(); // todo: change name
//                 if is_zstd {
//                     let Ok(tcp_fragment_decompressed) = zstd::decode_all(tcp_fragment_t) else {return};
//                     tcp_fragment.splice_remaining(&tcp_fragment_decompressed);
//                 }
//
//                 // recursively process the packet
//             }
//             _ => return,
//         }
//     }
// }

// todo: remove this test
#[cfg(test)]
mod tests {
    use crate::packets::opcodes::Pkt;
    use crate::packets::packet_process::process_packet;
    use crate::packets::utils::BinaryReader;

    #[tokio::test]
    async fn test_add() {
        use std::fs;
        let (packet_sender, _) = tokio::sync::mpsc::channel::<(Pkt, Vec<u8>)>(1);
        let filename = "src/packets/test_add_packet.json";
        let v: Vec<u8> = serde_json::from_str(&fs::read_to_string(filename).expect(&format!("Failed to open {filename}")))
            .expect("Invalid JSON in test_packet.json");
        process_packet(BinaryReader::from(v), packet_sender).await;
    }
}
