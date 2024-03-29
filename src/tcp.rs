use std::{
    collections::HashMap,
    net::{IpAddr, Ipv4Addr},
    sync::{Arc, Condvar, Mutex, RwLock},
};

use anyhow::{Context, Result};
use log::{debug, info, warn};
use pnet::{
    packet::{ip::IpNextHeaderProtocols, tcp::TcpPacket as PnetTcpPacket, Packet},
    transport::{self, TransportChannelType},
};
use rand::Rng;

use crate::{
    packet::{TcpFlags, TcpPacket},
    socket::{TcpSocket, TcpSocketId, TcpState},
};

const TCP_PORT_RANGE_START: u16 = 49152;
const TCP_PORT_RANGE_END: u16 = 65535;

const TCP_UNSPECIFIED_PORT: u16 = 0;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum TcpEvent {
    ConnectionEstablished(TcpSocketId),
}

impl std::fmt::Display for TcpEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::ConnectionEstablished(id) => format!("{} : ConnectionEstablished", id),
            }
        )
    }
}

pub struct TcpStream {
    sockets: RwLock<HashMap<TcpSocketId, TcpSocket>>,
    event_condvar: (Mutex<Option<TcpEvent>>, Condvar),
}

impl TcpStream {
    pub fn new() -> Arc<Self> {
        let sockets = RwLock::new(HashMap::new());
        let tcp = Arc::new(Self {
            sockets,
            event_condvar: (Mutex::new(None), Condvar::new()),
        });

        let cloned_tcp = Arc::clone(&tcp);
        std::thread::spawn(move || {
            // TODO: Handle error.
            cloned_tcp.receive_handler().unwrap();
        });

        tcp
    }

    /// Create a listening socket.
    pub fn listen(&self, local_address: Ipv4Addr, local_port: u16) -> Result<TcpSocketId> {
        let mut listening_socket = TcpSocket::new(
            local_address,
            local_port,
            // When a SYN packet is received, the remote address and port are assigned. See match arm TcpState::SynReceived in TcpSocket::receive_handler.
            Ipv4Addr::UNSPECIFIED,
            TCP_UNSPECIFIED_PORT,
        )?;
        listening_socket.state = TcpState::Listen;

        debug!("{} : Created a new listening socket", listening_socket.id());
        info!(
            "Listening on {}:{}",
            listening_socket.local_address, listening_socket.local_port
        );

        let mut socket_table = self
            .sockets
            .write()
            .map_err(|e| anyhow::anyhow!("{:?}", e))?;
        let socket_id = listening_socket.id();
        socket_table.insert(socket_id, listening_socket);

        Ok(socket_id)
    }

    /// Accept a connection that is established on a listening socket.
    pub fn accept(&self, listening_socket_id: TcpSocketId) -> Result<TcpSocketId> {
        self.wait_until(TcpEvent::ConnectionEstablished(listening_socket_id))?;

        let mut socket_table = self
            .sockets
            .write()
            .map_err(|e| anyhow::anyhow!("{:?}", e))?;

        let connected_socket_id = socket_table
            .get_mut(&listening_socket_id)
            .context(format!("No such listening socket {}", listening_socket_id))?
            .connected_socket_queue
            .pop_front()
            .context(format!("No connected socket on {}", listening_socket_id))?;

        info!("{} : Accepted the connection request.", connected_socket_id);

        Ok(connected_socket_id)
    }

    /// Create a new TCP socket and try to connect to the remote address.
    pub fn connect(&self, remote_address: Ipv4Addr, remote_port: u16) -> Result<TcpSocketId> {
        let mut socket = TcpSocket::new(
            // FIXME: Find the local address that is assigned to the interface that has the route to the remote address.
            // Since IP is not implemented, it may be necessary to refer to the kernel routing table. rtnetlink?
            "10.0.0.1".parse::<Ipv4Addr>()?,
            self.select_unused_port()?,
            remote_address,
            remote_port,
        )?;

        debug!("{} : Created a new TCP socket", socket.id());
        info!(
            "{} : Attempting to establish a TCP connection...",
            socket.id()
        );

        // SYN: First step of the three-way handshake.
        // FIXME: The iss must be a value that depends on a fictious 32-bit clock.
        // ref. 3.3. Sequence Numbers, Initial Sequence Number Selection
        let initial_sequence_number = rand::thread_rng().gen_range(0..(1 << 31));
        socket.snd.una = initial_sequence_number;
        socket.snd.nxt = initial_sequence_number + 1;
        socket.snd.iss = initial_sequence_number;
        // REVIEW: confirm with spec. snd.{up,wl1,wl2} and rcv.{nxt,up,iss} is initialized with 0. Is it okay?

        debug!("{} : SYN sent.", socket.id());
        socket.send_tcp_packet(socket.snd.iss, 0, TcpFlags::SYN, &[])?;
        debug!(
            "{} : State changed from {:?} to SynSent.",
            socket.id(),
            socket.state
        );
        socket.state = TcpState::SynSent;

        let mut socket_table = self
            .sockets
            .write()
            .map_err(|e| anyhow::anyhow!("{:?}", e))?;

        let socket_id = socket.id();
        socket_table.insert(socket.id(), socket);

        // To allow the receiving thread to acquire the lock.
        drop(socket_table);

        // Since we sent the SYN packet as the first step of 3-way handshake, we wait for the receiving thread to receive a SYN|ACK and send an ACK.
        self.wait_until(TcpEvent::ConnectionEstablished(socket_id))?;

        info!("{} : Connection established", socket_id);

        Ok(socket_id)
    }

    /// Select an unused local port in the range 49152..=65535.
    fn select_unused_port(&self) -> Result<u16> {
        let mut rng = rand::thread_rng();

        for _ in 0..(TCP_PORT_RANGE_END - TCP_PORT_RANGE_START + 1) {
            // Generate a random number in the range 49152..=65535.
            let local_port_candidate = rng.gen_range(TCP_PORT_RANGE_START..TCP_PORT_RANGE_END);
            let socket_table = match self.sockets.read() {
                Ok(socket_table) => socket_table,
                Err(_) => continue,
            };

            // Check if the candidate is already used.
            if socket_table
                .keys()
                .all(|id| id.local_port != local_port_candidate)
            {
                return Ok(local_port_candidate);
            }
        }

        Err(anyhow::anyhow!("No available local port"))
    }

    /// Wait until the specified event occurs.
    fn wait_until(&self, event: TcpEvent) -> Result<()> {
        let (lock, cvar) = &self.event_condvar;
        let mut notified_event = lock.lock().map_err(|e| anyhow::anyhow!("{:?}", e))?;

        debug!("{} waiting...", event);
        loop {
            if let Some(e) = *notified_event {
                if e == event {
                    break;
                }
            }

            notified_event = cvar
                .wait(notified_event)
                .map_err(|e| anyhow::anyhow!("{:?}", e))?;
        }

        debug!("{} event notified.", notified_event.unwrap());

        *notified_event = None;

        Ok(())
    }

    /// Notify the specified event.
    fn notify_event(&self, event: TcpEvent) -> Result<()> {
        let (lock, cvar) = &self.event_condvar;

        let mut notified_event = lock.lock().map_err(|e| anyhow::anyhow!("{:?}", e))?;
        *notified_event = Some(event);
        cvar.notify_all();

        Ok(())
    }

    /// Receive IPv4 packets and process them as TCP packets.
    pub fn receive_handler(&self) -> Result<()> {
        debug!("Recieving thread started.");
        info!("Started receiving arriving IP packets.");
        let (_, mut receiver) = transport::transport_channel(
            1 << 16,
            TransportChannelType::Layer3(IpNextHeaderProtocols::Tcp),
        )?;

        let mut ipv4_packet_iter = transport::ipv4_packet_iter(&mut receiver);

        // For each loop,
        // 1. Receive a IP packet.
        //     - Ignore IPv6 packets.
        // 2. Create a TCP packet from the payload of the IP packet.
        //     - pnet::Ipv4Packet -> pnet::TcpPacket -> toytcp::TcpPacket
        //     - For now, pnet parses the IP packet.
        // 3. Verify the TCP packet.
        // 4. Process the TCP packet.
        loop {
            // 1. Receive a IP packet.
            let Ok((packet, remote_address)) = ipv4_packet_iter.next() else {continue;};
            debug!("Received a IPv4 packet {:X?}.", &packet);

            // Ignore IPv6 packets.
            let IpAddr::V4(remote_address) = remote_address else {
                debug!("Ignored an IPv6 packet {:X?}", &packet);
                continue;
            };

            let local_address = packet.get_destination();

            // 2. Create a TCP packet from the payload of the IP packet.
            // Create a pnet::TcpPacket from the payload of the IPv4 packet.
            // TODO: Parse the IPv4 packet without pnet's help.
            let Some(packet) = PnetTcpPacket::new(packet.payload()) else {continue;};

            // Convert the pnet::TcpPacket to toytcp::TcpPacket.
            let packet = TcpPacket::from(packet);

            // 3. Verify the TCP packet.
            if !packet.verify_packet(local_address, remote_address) {
                warn!("Verification failed for the TCP packet {:X?}", &packet);
                continue;
            }
            debug!("Verified the TCP packet {:X?}", &packet);

            // 4. Process the TCP packet.
            let mut socket_table = self.sockets.write().unwrap();
            // This socket is either a connected socket or a listening socket.
            let socket = match socket_table.get_mut(&TcpSocketId {
                local_address,
                local_port: packet.destination_port(),
                remote_address,
                remote_port: packet.source_port(),
            }) {
                // Connected socket.
                Some(connected_socket) => connected_socket,
                None => {
                    match socket_table.get_mut(&TcpSocketId {
                        local_address,
                        local_port: packet.destination_port(),
                        remote_address: Ipv4Addr::UNSPECIFIED,
                        remote_port: TCP_UNSPECIFIED_PORT,
                    }) {
                        // Listening socket.
                        Some(listening_socket) => listening_socket,
                        // No corresponding socket.
                        None => continue,
                    }
                }
            };

            match socket.state {
                TcpState::Listen => {
                    if packet.flags() == TcpFlags::ACK {
                        // Originally send RST.
                        continue;
                    }

                    // In response to a received SYN on the listening socket, send SYN|ACK.
                    if packet.flags() == TcpFlags::SYN {
                        info!(
                            "{} : Received a connection request on the listening socket.",
                            socket.id()
                        );
                        debug!("{} : SYN received on the listening socket.", socket.id());
                        let listening_socket = &*socket;
                        let remote_port = packet.source_port();
                        // The socket transitions to a 'connected socket' upon receiving an ACK.
                        let mut connected_socket = TcpSocket::new(
                            listening_socket.local_address,
                            listening_socket.local_port,
                            remote_address,
                            remote_port,
                        )?;
                        debug!(
                            "{} : State changed from {:?} to SynReceived.",
                            connected_socket.id(),
                            connected_socket.state
                        );
                        connected_socket.state = TcpState::SynReceived;
                        // TODO: Output the current state in debug!s.
                        debug!(
                            "{} : Created a new connected socket.",
                            connected_socket.id(),
                        );

                        connected_socket.rcv.nxt = packet.sequence_number() + 1;
                        connected_socket.rcv.irs = packet.sequence_number();

                        connected_socket.snd.iss = rand::thread_rng().gen_range(0..(1 << 31));
                        connected_socket.snd.una = connected_socket.snd.iss;
                        connected_socket.snd.nxt = connected_socket.snd.iss + 1;
                        connected_socket.snd.wnd = packet.window_size();

                        debug!("{} : SYN|ACK sent.", connected_socket.id());
                        connected_socket.send_tcp_packet(
                            connected_socket.snd.iss,
                            connected_socket.rcv.nxt,
                            TcpFlags::SYN | TcpFlags::ACK,
                            &[],
                        )?;

                        connected_socket.listening_socket = Some(listening_socket.id());
                        socket_table.insert(connected_socket.id(), connected_socket);
                    }
                }
                TcpState::SynSent => {
                    if packet.flags() == TcpFlags::SYN | TcpFlags::ACK
                        // SND.UNA <= SEG.ACK <= SND.NXT.
                        && socket.snd.una <= packet.acknowledgment_number()
                        && packet.acknowledgment_number() <= socket.snd.nxt
                    {
                        debug!("{} : SYN|ACK received.", socket.id());
                        // Processing for <-- ACK.
                        socket.snd.una = packet.acknowledgment_number();
                        socket.snd.wnd = packet.window_size();

                        // Processing for <-- SYN
                        socket.rcv.nxt = packet.sequence_number() + 1;
                        socket.rcv.irs = packet.sequence_number();

                        // Basic 3-way handshake.
                        // see Section 3.4. Establishing a Connection - Figure 8.
                        if socket.snd.iss < socket.snd.una {
                            debug!("{} : ACK sent.", socket.id());
                            socket.send_tcp_packet(
                                socket.snd.nxt,
                                socket.rcv.nxt,
                                TcpFlags::ACK,
                                &[],
                            )?;
                            debug!(
                                "{} : State changed from {:?} to Established.",
                                socket.id(),
                                socket.state
                            );
                            socket.state = TcpState::Established;
                            self.notify_event(TcpEvent::ConnectionEstablished(socket.id()))?;
                        }
                        // Simultaneous 3-way handshake.
                        // see Section 3.4. Establishing a Connection - Figure 9.
                        else {
                            debug!(
                                "{} : State changed from {:?} to SynReceived.",
                                socket.id(),
                                socket.state
                            );
                            socket.state = TcpState::SynReceived;
                            socket.send_tcp_packet(
                                socket.snd.iss,
                                socket.rcv.nxt,
                                TcpFlags::ACK,
                                &[],
                            )?;
                        }
                    }
                }
                TcpState::SynReceived => {
                    let connected_socket = socket;

                    if packet.flags() == TcpFlags::ACK
                        && connected_socket.snd.una <= packet.acknowledgment_number()
                        && packet.acknowledgment_number() <= connected_socket.snd.nxt
                    {
                        debug!("{} : ACK received.", connected_socket.id());
                        connected_socket.rcv.nxt = packet.sequence_number() + 1;
                        connected_socket.snd.una = packet.acknowledgment_number();

                        debug!(
                            "{} : State changed from {:?} to Established.",
                            connected_socket.id(),
                            connected_socket.state
                        );
                        connected_socket.state = TcpState::Established;

                        let connected_socket_id = connected_socket.id();

                        if let Some(listening_socket_id) = connected_socket.listening_socket {
                            debug!(
                                "{} : Enqueued to the connected socket queue.",
                                connected_socket.id()
                            );
                            let listening_socket =
                                socket_table.get_mut(&listening_socket_id).unwrap();
                            listening_socket
                                .connected_socket_queue
                                .push_back(connected_socket_id);
                            // By notifying here, the accept() can dequeue the connected socket and return it to the user.
                            debug!(
                                "{} : Notification sent, ready accept() to dequeue the connected socket.",
                                connected_socket_id
                            );
                            self.notify_event(TcpEvent::ConnectionEstablished(
                                listening_socket_id,
                            ))?;
                        }
                    }
                }
                _ => todo!("no implentation for state {:?}", socket.state),
            }
        }
    }
}
