use std::net::Ipv4Addr;

use pnet::{
    packet::{ip::IpNextHeaderProtocols, tcp::TcpPacket as PnetTcpPakcet, Packet},
    util,
};

// No Options field for simplicity.
const TCP_HEADER_SIZE: usize = 20;

/// Control bits.
#[allow(non_snake_case)]
pub mod TcpFlags {
    /// Urgent Pointer field significant.
    pub const URG: u8 = 0b10_0000;
    /// Acknowledgment field significant.
    pub const ACK: u8 = 0b01_0000;
    /// Push Function.
    pub const PSH: u8 = 0b00_1000;
    /// Reset the connection.
    pub const RST: u8 = 0b00_0100;
    /// Synchronize sequence numbers.
    pub const SYN: u8 = 0b00_0010;
    /// No more data from sender.
    pub const FIN: u8 = 0b00_0001;
}

/// TCP packet.
#[derive(Debug)]
pub struct TcpPacket {
    buffer: Vec<u8>,
}

impl TcpPacket {
    pub fn new(payload_size: usize) -> Self {
        Self {
            buffer: vec![0; TCP_HEADER_SIZE + payload_size],
        }
    }

    // Getters
    pub fn source_port(&self) -> u16 {
        u16::from_be_bytes([self.buffer[0], self.buffer[1]])
    }

    pub fn destination_port(&self) -> u16 {
        u16::from_be_bytes([self.buffer[2], self.buffer[3]])
    }

    pub fn sequence_number(&self) -> u32 {
        u32::from_be_bytes([
            self.buffer[4],
            self.buffer[5],
            self.buffer[6],
            self.buffer[7],
        ])
    }

    pub fn acknowledgment_number(&self) -> u32 {
        u32::from_be_bytes([
            self.buffer[8],
            self.buffer[9],
            self.buffer[10],
            self.buffer[11],
        ])
    }

    pub fn data_offset(&self) -> u8 {
        self.buffer[12] >> 4
    }

    pub fn flags(&self) -> u8 {
        self.buffer[13]
    }

    pub fn window_size(&self) -> u16 {
        u16::from_be_bytes([self.buffer[14], self.buffer[15]])
    }

    pub fn checksum(&self) -> u16 {
        u16::from_be_bytes([self.buffer[16], self.buffer[17]])
    }

    pub fn urgent_pointer(&self) -> u16 {
        u16::from_be_bytes([self.buffer[18], self.buffer[19]])
    }

    // Setters
    pub fn set_source_port(&mut self, source_port: u16) {
        self.buffer[0..2].copy_from_slice(&source_port.to_be_bytes());
    }

    pub fn set_destination_port(&mut self, destination_port: u16) {
        self.buffer[2..4].copy_from_slice(&destination_port.to_be_bytes());
    }

    pub fn set_sequence_number(&mut self, sequence_number: u32) {
        self.buffer[4..8].copy_from_slice(&sequence_number.to_be_bytes());
    }

    pub fn set_acknowledgment_number(&mut self, acknowledgment_number: u32) {
        self.buffer[8..12].copy_from_slice(&acknowledgment_number.to_be_bytes());
    }

    pub fn set_data_offset(&mut self, data_offset: u8) {
        self.buffer[12] |= data_offset << 4;
    }

    pub fn set_flags(&mut self, flags: u8) {
        self.buffer[13] = flags;
    }

    pub fn set_window_size(&mut self, window_size: u16) {
        self.buffer[14..16].copy_from_slice(&window_size.to_be_bytes());
    }

    pub fn set_checksum(&mut self, checksum: u16) {
        self.buffer[16..18].copy_from_slice(&checksum.to_be_bytes());
    }

    pub fn set_urgent_pointer(&mut self, urgent_pointer: u16) {
        self.buffer[18..20].copy_from_slice(&urgent_pointer.to_be_bytes());
    }

    pub fn set_payload(&mut self, payload: &[u8]) {
        assert_eq!(TCP_HEADER_SIZE + payload.len(), self.buffer.len());
        self.buffer[TCP_HEADER_SIZE..].copy_from_slice(payload);
    }

    /// Verify checksum of the TCP packet.
    pub fn verify_packet(&self, local_address: Ipv4Addr, remote_address: Ipv4Addr) -> bool {
        self.checksum()
            == util::ipv4_checksum(
                self.packet(),
                8,
                &[],
                &local_address,
                &remote_address,
                IpNextHeaderProtocols::Tcp,
            )
    }
}

impl pnet::packet::Packet for TcpPacket {
    fn packet(&self) -> &[u8] {
        &self.buffer
    }

    fn payload(&self) -> &[u8] {
        &self.buffer[TCP_HEADER_SIZE..]
    }
}

impl<'a> From<PnetTcpPakcet<'a>> for TcpPacket {
    fn from(packet: PnetTcpPakcet) -> Self {
        Self {
            buffer: packet.packet().to_vec(),
        }
    }
}
