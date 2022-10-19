use crate::packet::TcpPacket;

use std::net::{IpAddr, Ipv4Addr};

use anyhow::Result;
use pnet::packet::ip::IpNextHeaderProtocols;
use pnet_transport::{self, TransportChannelType, TransportProtocol, TransportSender};

pub struct Socket {
    src_addr: Ipv4Addr,
    dst_addr: Ipv4Addr,
    src_port: u16,
    dst_port: u16,
    sender: TransportSender,
}

impl Socket {
    pub fn new(
        src_addr: Ipv4Addr,
        dst_addr: Ipv4Addr,
        src_port: u16,
        dst_port: u16,
    ) -> Result<Self> {
        let (sender, _) = pnet_transport::transport_channel(
            65535,
            TransportChannelType::Layer4(TransportProtocol::Ipv4(IpNextHeaderProtocols::Tcp)),
        )?;
        Ok(Self {
            src_addr,
            dst_addr,
            src_port,
            dst_port,
            sender,
        })
    }

    pub fn send_tcp_packet(&mut self, flag: u8, payload: &[u8]) -> Result<usize> {
        let packet = TcpPacket::new(self.src_port, self.dst_port, flag, payload);
        let len = self.sender.send_to(packet, IpAddr::V4(self.dst_addr))?;
        Ok(len)
    }
}
