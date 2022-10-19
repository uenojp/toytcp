use crate::tcp::TCP_HEADER_SIZE;

use pnet::packet::Packet;

#[derive(Clone)]
pub struct TcpPacket {
    buffer: Vec<u8>,
}

impl TcpPacket {
    pub fn new(src_port: u16, dst_port: u16, flag: u8, payload: &[u8]) -> Self {
        let mut buffer = vec![0; TCP_HEADER_SIZE + payload.len()];
        buffer[0..2].copy_from_slice(&src_port.to_be_bytes());
        buffer[2..4].copy_from_slice(&dst_port.to_be_bytes());
        buffer[13] = flag;
        Self { buffer }
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
