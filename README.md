# toytcp

A subset implementation of TCP based on RFC793.

Currently only active open is implemented.

Inspired by
- https://techbookfest.org/product/6562563816947712
- https://github.com/teru01/toytcp

## setup

On Ubuntu 22.04 LTS.

```bash
sudo apt install ethtool
# Setup network namespace.
./setup.sh
sudo ip netns exec host1 wireshark
# Start server.
sudo ip netns exec host2 nc -l 10.0.1.1 40000
# Start client.
cargo build --example echoclient
sudo ip netns exec host1 ./target/debug/examples/echoclient 10.0.1.1 40000 
```

### log
```
[2023-08-22T09:34:35Z DEBUG toytcp::tcp] Recieving thread started.
[2023-08-22T09:34:35Z INFO  toytcp::tcp] Listening for incoming IPv4 packets...
[2023-08-22T09:34:35Z DEBUG toytcp::tcp] 10.0.0.1:54677 -> 10.0.1.1:40000 : Created a new TCP socket
[2023-08-22T09:34:35Z INFO  toytcp::tcp] 10.0.0.1:54677 -> 10.0.1.1:40000 : Attempting to establish a TCP connection...
[2023-08-22T09:34:35Z DEBUG toytcp::tcp] 10.0.0.1:54677 -> 10.0.1.1:40000 : SYN sent.
[2023-08-22T09:34:35Z DEBUG toytcp::socket] 10.0.0.1:54677 -> 10.0.1.1:40000 : Sent 20 bytes [D5, 95, 9C, 40, 64, 6A, 15, 64, 0, 0, 0, 0, 50, 2, 11, 1C, 9E, 20, 0, 0]
[2023-08-22T09:34:35Z DEBUG toytcp::tcp] Received a IPv4 packet Ipv4Packet { version : 4, header_length : 5, dscp : 0, ecn : 0, total_length : 44, identification : 0, flags : 2, fragment_offset : 0, ttl : 63, next_level_protocol : IpNextHeaderProtocol(6), checksum : 9931, source : 10.0.1.1, destination : 10.0.0.1, options : [],  }.
[2023-08-22T09:34:35Z DEBUG toytcp::tcp] 10.0.0.1:54677 -> 10.0.1.1:40000 : ConnectionEstablished waiting...
[2023-08-22T09:34:35Z DEBUG toytcp::tcp] 10.0.0.1:54677 -> 10.0.1.1:40000 : Verified the TCP packet TcpPacket { buffer: [9C, 40, D5, 95, 63, 2F, 1, 20, 64, 6A, 15, 65, 60, 12, FA, F0, 38, 2F, 0, 0, 2, 4, 5, B4] }
[2023-08-22T09:34:35Z DEBUG toytcp::tcp] 10.0.0.1:54677 -> 10.0.1.1:40000 : SYN|ACK received.
[2023-08-22T09:34:35Z DEBUG toytcp::tcp] 10.0.0.1:54677 -> 10.0.1.1:40000 : ACK sent.
[2023-08-22T09:34:35Z DEBUG toytcp::socket] 10.0.0.1:54677 -> 10.0.1.1:40000 : Sent 20 bytes [D5, 95, 9C, 40, 64, 6A, 15, 65, 63, 2F, 1, 21, 50, 10, 11, 1C, 39, C1, 0, 0]
[2023-08-22T09:34:35Z DEBUG toytcp::tcp] 10.0.0.1:54677 -> 10.0.1.1:40000 : ConnectionEstablished event notified.
[2023-08-22T09:34:35Z INFO  toytcp::tcp] 10.0.0.1:54677 -> 10.0.1.1:40000 : Connection established
```
