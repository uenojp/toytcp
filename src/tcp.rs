use crate::socket::Socket;

use std::net::Ipv4Addr;

use anyhow::Result;

pub const TCP_HEADER_SIZE: usize = 20;

pub mod ControlFlags {
    pub const FIN: u8 = 1 << 0; // No more data from sender
    pub const SYN: u8 = 1 << 1; // Synchronize sequence numbers
    pub const RST: u8 = 1 << 2; // Reset the connection
    pub const PSH: u8 = 1 << 3; // Push Function
    pub const ACK: u8 = 1 << 4; // Acknowledgment field significant
    pub const URG: u8 = 1 << 5; // Urgent Pointer field significant
    pub const ECE: u8 = 1 << 6; // ECN-Echo
    pub const CWR: u8 = 1 << 7; // Congestion Window Reduced
}

pub struct Tcp;

impl Tcp {
    pub fn new() -> Self {
        Self {}
    }

    pub fn connect(&self, addr: Ipv4Addr, port: u16) -> Result<()> {
        let mut socket = Socket::new(Ipv4Addr::new(10, 0, 0, 1), addr, 33333, port)?;
        socket.send_tcp_packet(ControlFlags::SYN, &[])?;
        Ok(())
    }
}
