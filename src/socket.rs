use std::net::{IpAddr, Ipv4Addr};

use anyhow::{Context, Result};
use log::debug;
use pnet::{
    packet::{ip::IpNextHeaderProtocols, Packet},
    transport::{self, TransportChannelType, TransportProtocol, TransportSender},
};

use crate::packet::TcpPacket;

const TCP_SOCKET_BUFFER_SIZE: usize = 4380;

/// Four-tuple uniquely identifying a TCP socket.
#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub struct TcpSocketId {
    pub local_address: Ipv4Addr,
    pub local_port: u16,
    pub remote_address: Ipv4Addr,
    pub remote_port: u16,
}

impl std::fmt::Display for TcpSocketId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}:{} -> {}:{}",
            self.local_address, self.local_port, self.remote_address, self.remote_port
        )
    }
}

/// Send Sequence Variables.
#[derive(Debug)]
pub struct SendSequenceVariables {
    /// Send unacknowledged.
    pub una: u32,
    /// Send next.
    pub nxt: u32,
    /// Send window.
    pub wnd: u16,
    /// Send urgent pointer.
    pub up: u16,
    /// Segment sequence number used for last window update.
    pub wl1: u32,
    /// Segment acknowledgment number used for last window update.
    pub wl2: u32,
    /// Initial send sequence number.
    pub iss: u32,
}

impl SendSequenceVariables {
    pub fn with_window_size(wnd: u16) -> Self {
        Self {
            una: 0,
            nxt: 0,
            wnd,
            up: 0,
            wl1: 0,
            wl2: 0,
            iss: 0,
        }
    }
}

/// Receive Sequence Variables.
#[derive(Debug)]
pub struct ReceiveSequenceVariables {
    /// Receive next.
    pub nxt: u32,
    /// Receive window.
    pub wnd: u16,
    /// Receive urgent pointer.
    pub up: u16,
    /// Initial receive sequence number.
    pub irs: u32,
}

impl ReceiveSequenceVariables {
    pub fn with_window_size(wnd: u16) -> Self {
        Self {
            nxt: 0,
            wnd,
            up: 0,
            irs: 0,
        }
    }
}

/// TCP state.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum TcpState {
    Listen,
    SynSent,
    SynReceived,
    Established,
    FinWait1,
    FinWait2,
    CloseWait,
    Closing,
    LastAck,
    TimeWait,
    Closed,
}

/// TCP socket.
pub struct TcpSocket {
    pub local_address: Ipv4Addr,
    pub local_port: u16,
    pub remote_address: Ipv4Addr,
    pub remote_port: u16,
    pub snd: SendSequenceVariables,
    pub rcv: ReceiveSequenceVariables,
    pub state: TcpState,
    /// A transmission channel.
    /// This channel uses a raw socket. When a TCP packet is written to this channel, it is transmitted
    /// with an IP header.
    sender: TransportSender,
}

impl TcpSocket {
    pub fn new(
        local_address: Ipv4Addr,
        local_port: u16,
        remote_address: Ipv4Addr,
        remote_port: u16,
    ) -> Result<Self> {
        let (sender, _) = transport::transport_channel(
            1 << 16,
            TransportChannelType::Layer4(TransportProtocol::Ipv4(IpNextHeaderProtocols::Tcp)),
        )?;

        Ok(Self {
            local_address,
            local_port,
            remote_address,
            remote_port,
            snd: SendSequenceVariables::with_window_size(TCP_SOCKET_BUFFER_SIZE as u16),
            rcv: ReceiveSequenceVariables::with_window_size(TCP_SOCKET_BUFFER_SIZE as u16),
            state: TcpState::Closed,
            sender,
        })
    }

    /// Send a TCP packet.
    pub fn send_tcp_packet(
        &mut self,
        sequence_number: u32,
        acknowledgment_number: u32,
        flags: u8,
        payload: &[u8],
    ) -> Result<usize> {
        let mut packet = TcpPacket::new(payload.len());
        packet.set_source_port(self.local_port);
        packet.set_destination_port(self.remote_port);
        packet.set_sequence_number(sequence_number);
        packet.set_acknowledgment_number(acknowledgment_number);
        packet.set_data_offset(5); // = TCP_HEADER_SIZE / 4
        packet.set_flags(flags);
        packet.set_window_size(self.rcv.wnd);
        packet.set_payload(payload);
        // TODO: Calculate the checksum without pnet's help.
        packet.set_checksum(pnet::util::ipv4_checksum(
            packet.packet(),
            8,
            &[],
            &self.local_address,
            &self.remote_address,
            IpNextHeaderProtocols::Tcp,
        ));

        let sent_size = self
            .sender
            .send_to(&packet, IpAddr::V4(self.remote_address))
            .context(format!(
                "{} : Failed to send the packet {:X?}",
                self.id(),
                &packet.packet()
            ))?;

        debug!(
            "{} : Sent {} bytes {:X?}",
            self.id(),
            sent_size,
            &packet.packet()
        );

        Ok(sent_size)
    }

    /// Get a four-tuple uniquely identifying this socket.
    pub fn id(&self) -> TcpSocketId {
        TcpSocketId {
            local_address: self.local_address,
            local_port: self.local_port,
            remote_address: self.remote_address,
            remote_port: self.remote_port,
        }
    }
}
