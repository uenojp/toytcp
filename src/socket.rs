use crate::{packet::TcpPacket, tcp::TCP_HEADER_SIZE};

use std::net::{IpAddr, Ipv4Addr};

use anyhow::Result;
use pnet::packet::ip::IpNextHeaderProtocols;
use pnet_transport::{self, TransportChannelType, TransportProtocol, TransportSender};

pub struct Socket {
    source_address: Ipv4Addr,
    destination_address: Ipv4Addr,
    source_port: u16,
    destination_port: u16,
    sender: TransportSender,
}

impl Socket {
    pub fn new(
        source_address: Ipv4Addr,
        destination_address: Ipv4Addr,
        source_port: u16,
        destination_port: u16,
    ) -> Result<Self> {
        let (sender, _) = pnet_transport::transport_channel(
            65535,
            TransportChannelType::Layer4(TransportProtocol::Ipv4(IpNextHeaderProtocols::Tcp)),
        )?;
        Ok(Self {
            source_address,
            destination_address,
            source_port,
            destination_port,
            sender,
        })
    }

    pub fn send_tcp_packet(&mut self, flag: u8, payload: &[u8]) -> Result<usize> {
        let mut packet = TcpPacket::new();
        packet.set_source(self.source_port);
        packet.set_destination(self.destination_port);
        packet.set_sequence(0);
        packet.set_acknowlegement(0);
        packet.set_data_offset((TCP_HEADER_SIZE / 4) as u8);
        packet.set_flag(flag);
        packet.set_window(0);
        packet.set_checksum(0);
        packet.set_payload(payload);

        let len = self
            .sender
            .send_to(packet, IpAddr::V4(self.destination_address))?;
        Ok(len)
    }
}
