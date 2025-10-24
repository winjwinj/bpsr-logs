use byteorder::{BigEndian, ReadBytesExt};
use std::collections::BTreeMap;
use std::io::{Cursor, Read};
use std::{fmt, io};

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Server {
    src_addr: [u8; 4],
    src_port: u16,
    dst_addr: [u8; 4],
    dst_port: u16,
}

impl Server {
    pub fn new(src_addr: [u8; 4], src_port: u16, dst_addr: [u8; 4], dst_port: u16) -> Self {
        Self {
            src_addr,
            src_port,
            dst_addr,
            dst_port,
        }
    }
}

impl fmt::Display for Server {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}:{} -> {}:{}",
            ip_to_str(&self.src_addr),
            self.src_port,
            ip_to_str(&self.dst_addr),
            self.dst_port
        )
    }
}

fn ip_to_str(ip: &[u8; 4]) -> String {
    format!("{}.{}.{}.{}", ip[0], ip[1], ip[2], ip[3])
}

pub struct TCPReassembler {
    pub cache: BTreeMap<usize, Vec<u8>>, // sequence -> payload
    pub next_seq: Option<usize>,         // next expected sequence
    pub _data: Vec<u8>,
}

impl TCPReassembler {
    pub fn new() -> Self {
        Self {
            cache: BTreeMap::new(),
            next_seq: None,
            _data: Vec::new(),
        }
    }

    // // Push a TCP segment and try to reassemble contiguous data.
    // /// Returns Some(Vec<u8>) if contiguous data is available, None otherwise.
    // pub fn push_segment(&mut self, packet: TcpSlice) -> Option<(usize, Vec<u8>)> {
    //     let payload = packet.payload().to_vec();
    //     let seq = packet.sequence_number() as usize;
    //     if payload.is_empty() {
    //         return None;
    //     }
    //
    //     // Insert segment into cache
    //     self.cache.insert(seq, payload);
    //
    //     // Initialize next_seq to the lowest sequence seen if not set
    //     if self.next_seq.is_none() {
    //         if let Some((&lowest_seq, _)) = self.cache.first_key_value() {
    //             self.next_seq = Some(lowest_seq);
    //         }
    //     }
    //
    //     // Try to assemble contiguous data
    //     let mut output = Vec::new();
    //     while let Some(next) = self.next_seq {
    //         if let Some(segment) = self.cache.remove(&next) {
    //             // advance next_seq only when we actually use this segment
    //             self.next_seq = Some(next.wrapping_add(segment.len()));
    //             output.extend(segment);
    //         } else {
    //             break;
    //         }
    //     }
    //
    //     if output.is_empty() {
    //         None
    //     } else {
    //         Some((self.next_seq?, output))
    //     }
    // }

    pub fn clear_reassembler(&mut self, seq_number: usize) {
        self.cache = BTreeMap::new();
        self.next_seq = Some(seq_number)
    }
}

// Binary reader implementation
pub struct BinaryReader {
    pub cursor: Cursor<Vec<u8>>,
}

impl BinaryReader {
    pub fn from(data: Vec<u8>) -> Self {
        Self {
            cursor: Cursor::new(data),
        }
    }

    pub fn read_u16(&mut self) -> io::Result<u16> {
        self.cursor.read_u16::<BigEndian>()
    }

    pub fn read_u32(&mut self) -> io::Result<u32> {
        self.cursor.read_u32::<BigEndian>()
    }

    pub fn peek_u32(&mut self) -> io::Result<u32> {
        let pos = self.cursor.position();
        let value = self.cursor.read_u32::<BigEndian>()?;
        self.cursor.set_position(pos);
        Ok(value)
    }

    pub fn read_u64(&mut self) -> io::Result<u64> {
        self.cursor.read_u64::<BigEndian>()
    }

    pub fn read_string(&mut self) -> io::Result<String> {
        let mut s = String::new();
        self.cursor.read_to_string(&mut s)?;
        Ok(s)
    }

    pub fn read_bytes(&mut self, count: usize) -> io::Result<Vec<u8>> {
        let mut buffer = vec![0u8; count];
        self.cursor.read_exact(&mut buffer)?;
        Ok(buffer)
    }

    pub fn read_remaining(&mut self) -> &[u8] {
        let pos = self.cursor.position() as usize;
        let buf = self.cursor.get_ref();
        &buf[pos..]
    }

    pub fn remaining(&self) -> usize {
        let total_len = self.cursor.get_ref().len() as u64;
        let current_pos = self.cursor.position();
        (total_len.saturating_sub(current_pos)) as usize
    }

    pub const fn len(&self) -> usize {
        self.cursor.get_ref().len()
    }

    // pub fn splice_remaining(&mut self, data: &[u8]) {
    //     let start_range = self.cursor.position() as usize;
    //     let buf = self.cursor.get_mut();
    //     buf.splice(start_range.., data.iter().cloned());
    // }
}
