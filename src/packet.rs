use crate::tcp::TCP_HEADER_SIZE;

use pnet::packet::Packet;

// 
//     0                   1                   2                   3
//     0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
//    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
//    |          Source Port          |       Destination Port        |
//    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
//    |                        Sequence Number                        |
//    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
//    |                    Acknowledgment Number                      |
//    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
//    |  Data |           |U|A|P|R|S|F|                               |
//    | Offset| Reserved  |R|C|S|S|Y|I|            Window             |
//    |       |           |G|K|H|T|N|N|                               |
//    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
//    |           Checksum            |         Urgent Pointer        |
//    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
//    |                    Options                    |    Padding    |
//    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
//    |                             data                              |
//    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
// 
//                             TCP Header Format
//
// ref. https://www.rfc-editor.org/rfc/rfc793, Figure 3
//
#[derive(Clone)]
pub struct TcpPacket {
    buffer: Vec<u8>,
}

impl TcpPacket {
    pub fn new() -> Self {
        Self {
            buffer: vec![0; TCP_HEADER_SIZE],
        }
    }

    pub fn set_source(&mut self, source: u16) {
        self.buffer[0..2].copy_from_slice(&source.to_be_bytes());
    }

    pub fn set_destination(&mut self, destination: u16) {
        self.buffer[2..4].copy_from_slice(&destination.to_be_bytes());
    }

    pub fn set_sequence(&mut self, sequence: u32) {
        self.buffer[4..8].copy_from_slice(&sequence.to_be_bytes());
    }

    pub fn set_acknowlegement(&mut self, acknowlegement: u32) {
        self.buffer[8..12].copy_from_slice(&acknowlegement.to_be_bytes());
    }

    pub fn set_data_offset(&mut self, data_offset: u8) {
        self.buffer[12] |= data_offset << 4;
    }

    pub fn set_flag(&mut self, flag: u8) {
        self.buffer[13] = flag;
    }

    pub fn set_window(&mut self, window: u16) {
        self.buffer[14..16].copy_from_slice(&window.to_be_bytes());
    }

    pub fn set_checksum(&mut self, checksum: u16) {
        self.buffer[16..18].copy_from_slice(&checksum.to_be_bytes());
    }

    pub fn set_payload(&mut self, payload: &[u8]) {
        self.buffer.resize(TCP_HEADER_SIZE + payload.len(), 0);
        self.buffer[TCP_HEADER_SIZE..TCP_HEADER_SIZE + payload.len()].copy_from_slice(payload);
    }

    pub fn source(&self) -> u16 {
        u16::from_be_bytes([self.buffer[0], self.buffer[1]])
    }

    pub fn destination(&self) -> u16 {
        u16::from_be_bytes([self.buffer[2], self.buffer[3]])
    }

    pub fn sequence(&self) -> u32 {
        u32::from_be_bytes([
            self.buffer[4],
            self.buffer[5],
            self.buffer[6],
            self.buffer[7],
        ])
    }

    pub fn acknowlegement(&self) -> u32 {
        u32::from_be_bytes([
            self.buffer[8],
            self.buffer[9],
            self.buffer[10],
            self.buffer[11],
        ])
    }

    pub fn flag(&self) -> u8 {
        self.buffer[13]
    }

    pub fn window(&self) -> u16 {
        u16::from_be_bytes([self.buffer[14], self.buffer[15]])
    }

    pub fn checksum(&self) -> u16 {
        u16::from_be_bytes([self.buffer[16], self.buffer[17]])
    }
}

impl Packet for TcpPacket {
    fn packet(&self) -> &[u8] {
        &self.buffer
    }

    fn payload(&self) -> &[u8] {
        &self.buffer[TCP_HEADER_SIZE..]
    }
}
